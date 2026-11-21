use super::MAX_TASKS;

/// A task's index in the executor's task table, which its wakers carry.
#[derive(Clone, Copy)]
pub struct Slot(usize);

impl Slot {
    pub fn new(index: usize) -> Self {
        debug_assert!(index < MAX_TASKS, "a slot indexes the task table");
        Self(index)
    }

    pub fn index(self) -> usize { self.0 }

    /// The bitmap word that holds this slot's bit.
    pub fn word(self) -> usize { self.0 / u64::BITS as usize }

    /// This slot's bit in its word.
    pub fn bit(self) -> u64 { 1 << (self.0 % u64::BITS as usize) }
}
