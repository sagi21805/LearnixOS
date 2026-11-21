//! Runs the kernel's tasks, each polled only after its waker fires.

#[expect(dead_code, reason = "the executor takes ready slots in plan 06")]
pub mod ready;
#[expect(dead_code, reason = "the executor hands out slots in plan 06")]
pub mod slot;
pub mod task;
#[expect(dead_code, reason = "the executor polls with it in plan 06")]
mod waker;

/// The most tasks the executor runs at once.
pub const MAX_TASKS: usize = 512;

/// How many `u64` words a bitmap of every slot takes.
pub const WORDS: usize = MAX_TASKS / u64::BITS as usize;

const _: () = assert!(
    MAX_TASKS.is_multiple_of(u64::BITS as usize),
    "every word of a slot bitmap is whole"
);
