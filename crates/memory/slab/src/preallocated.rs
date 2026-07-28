use core::{fmt::Debug, mem::ManuallyDrop};
use nonmax::NonMaxU16;

/// Preallocated object in the slab allocator.
pub union PreAllocated<T: Sized> {
    pub allocated: ManuallyDrop<T>,
    pub next_free_idx: Option<NonMaxU16>,
}

impl<T: Debug> Debug for PreAllocated<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PreAllocated")
            .field("allocated", unsafe { &self.allocated })
            .field("next_free_idx", unsafe { &self.next_free_idx })
            .finish()
    }
}
