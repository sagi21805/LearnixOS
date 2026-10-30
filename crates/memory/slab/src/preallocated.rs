use core::{fmt::Debug, mem::ManuallyDrop};

use crate::{Generic64, slot_index::SlotIndex};

/// Preallocated object in the slab allocator.
///
/// It is `repr(C)`, so both fields start at the slot's first byte.
#[repr(C)]
pub union PreAllocated<T: Sized> {
    pub allocated: ManuallyDrop<T>,
    pub next_free_idx: Option<SlotIndex>,
}

// A slot is exactly as large as the object it holds.
const _: () = assert!(size_of::<PreAllocated<Generic64>>() == 64);

/// Prints no fields, as nothing in the slot says which one is live.
impl<T> Debug for PreAllocated<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PreAllocated").finish_non_exhaustive()
    }
}
