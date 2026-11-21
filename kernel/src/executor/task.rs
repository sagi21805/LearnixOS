use core::{
    pin::Pin,
    task::{Context, Poll},
};

use common::constants::REGULAR_PAGE_SIZE;
use slab::{
    StaticCacheAllocator, preallocated::PreAllocated, traits::SlabGeometry,
};

use crate::{
    console,
    executor::{Executor, TaskTable},
    memory::KernelSlab,
};

/// The kernel's executor, which boxes each task in `Task`'s slab cache.
pub type KernelExecutor = Executor<Task, TaskAllocator>;

/// Boxes a task in `Task`'s slab cache.
type TaskAllocator = StaticCacheAllocator<Task, KernelSlab>;

const _: () = assert!(
    size_of::<TaskTable<Task, TaskAllocator>>() == REGULAR_PAGE_SIZE,
    "the task table fills one page"
);
const _: () = assert!(
    size_of::<TaskTable<Task, TaskAllocator>>()
        > slab::LARGEST_GENERIC_SIZE,
    "the task table comes from buddy, not from a generic size"
);

/// Every task the kernel runs, each in a box from its own slab cache.
pub enum Task {
    Console(ConsoleTask),
}

/// The future of [`console::run`].
pub type ConsoleTask = impl Future<Output = ()>;

/// The console loop, ready to spawn as [`Task::Console`].
#[define_opaque(ConsoleTask)]
pub fn console_task() -> ConsoleTask { console::run() }

// `#[typestate_groups::state]` takes only structs.
impl typestate_groups::State for Task {}

impl Future for Task {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // SAFETY: a variant's future never moves out of `self`, so pinning
        // `self` pins it.
        unsafe {
            match self.get_unchecked_mut() {
                Task::Console(future) => {
                    Pin::new_unchecked(future).poll(cx)
                }
            }
        }
    }
}

slab::slab_family!(KernelTasks: Task);

// A new variant can't push the slab's waste above 1/8.
const _: () = {
    let used = Task::OBJECTS_PER_SLAB * size_of::<PreAllocated<Task>>();
    assert!(
        used * 8 >= Task::SLAB_LAYOUT.size() * 7,
        "a task's slab wastes at most 1/8 of its pages"
    );
};
