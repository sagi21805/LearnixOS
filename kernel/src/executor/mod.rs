//! Runs the kernel's tasks, each polled only after its waker fires.

use alloc::{boxed::Box, collections::VecDeque};
use core::{alloc::AllocError, pin::Pin, task::Context};

use common::constants::REGULAR_PAGE_SIZE;
use slab::SlabBoxNew;
use thiserror::Error;

use crate::{
    executor::{free::FreeSlots, ready::READY, slot::Slot, task::Task},
    memory::SlabBox,
};

mod free;
pub mod ready;
pub mod slot;
pub mod task;
mod waker;

/// The most tasks the executor runs at once.
pub const MAX_TASKS: usize = 512;

/// How many `u64` words a bitmap of every slot takes.
pub const WORDS: usize = MAX_TASKS / u64::BITS as usize;

const _: () = assert!(
    MAX_TASKS.is_multiple_of(u64::BITS as usize),
    "every word of a slot bitmap is whole"
);

/// Each slot's task, `None` while the slot is free.
type TaskTable = [Option<Pin<SlabBox<Task>>>; MAX_TASKS];

const _: () = assert!(
    size_of::<TaskTable>() == REGULAR_PAGE_SIZE,
    "the task table fills one page"
);
const _: () = assert!(
    size_of::<TaskTable>() > slab::LARGEST_GENERIC_SIZE,
    "the task table comes from buddy, not from a generic size"
);

/// Runs the kernel's tasks, polling each one only after its waker fires.
///
/// Wakers record ready slots in [`READY`]. Each round, the executor moves
/// them into its run queue and polls them in that order, so the run queue
/// is the scheduling policy.
pub struct Executor {
    /// Indexed by [`Slot`]. A task keeps its slot until it finishes.
    tasks: Box<TaskTable>,
    free: FreeSlots,
    /// The slots this round polls, in the order it polls them.
    run_queue: VecDeque<Slot>,
}

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("the executor runs at most {} tasks", MAX_TASKS)]
    Full,
    #[error("no slab slot is free for the task")]
    OutOfMemory(#[from] AllocError),
}

impl Executor {
    /// An executor with every slot free. Its table and run queue take one
    /// page each.
    pub fn new() -> Self {
        Self {
            tasks: Box::new([const { None }; MAX_TASKS]),
            free: FreeSlots::new(),
            run_queue: VecDeque::with_capacity(MAX_TASKS),
        }
    }

    /// Puts `task` in a free slot and marks it ready for its first poll.
    ///
    /// It allocates before it takes a slot, so a failed allocation leaves
    /// every slot as it was.
    pub fn spawn(&mut self, task: Task) -> Result<(), SpawnError> {
        let task = Box::into_pin(SlabBox::try_new_slab(task)?);
        let slot = self.free.take().ok_or(SpawnError::Full)?;
        self.tasks[slot.index()] = Some(task);
        READY.wake(slot);
        Ok(())
    }

    /// Polls ready tasks forever, halting while none is ready.
    ///
    /// The run queue is refilled only once it is empty, and one
    /// [`take`](ready::ReadySet::take) yields a slot at most once, so it
    /// never holds more than [`MAX_TASKS`] slots and never reallocates.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_queue.extend(READY.take());
            while let Some(slot) = self.run_queue.pop_front() {
                self.poll(slot);
            }
            READY.wait_for_task();
        }
    }

    /// Polls the task in `slot`, and frees it once it finishes.
    fn poll(&mut self, slot: Slot) {
        // A waker that outlived its task may mark a free slot.
        let Some(task) = &mut self.tasks[slot.index()] else {
            return;
        };
        let waker = slot.waker();
        let mut cx = Context::from_waker(&waker);
        if task.as_mut().poll(&mut cx).is_ready() {
            self.tasks[slot.index()] = None;
            self.free.put(slot);
        }
    }
}

impl Default for Executor {
    fn default() -> Self { Self::new() }
}
