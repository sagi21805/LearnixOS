use core::ptr::NonNull;

use common::constants::REGULAR_PAGE_ALIGNMENT;
use nonmax::NonMaxU16;

use crate::{preallocated::PreAllocated, traits::Slab};

/// The object slots of one slab, in pages that no other value manages.
///
/// It is neither `Copy` nor `Clone` and its pointer is private, so each
/// slab's pages have exactly one manager.
#[repr(transparent)]
pub struct SlabObjects<T: Slab>(NonNull<PreAllocated<T>>);

// SAFETY: `SlabObjects` is the only value that reaches its slab's free
// slots, because `new` requires pages nothing else points at, the pointer
// never leaves this type, and the type can't be copied or cloned.
// Allocated slots are reached through the pointers `kmalloc` hands out,
// and no method reads the `T` in them. So nothing ties a `SlabObjects` to
// one thread, whatever `T` is.
unsafe impl<T: Slab> Send for SlabObjects<T> {}

impl<T: Slab> SlabObjects<T> {
    /// Takes over the `T::PAGES_PER_SLAB` pages at `pages`.
    ///
    /// # Safety
    ///
    /// `pages` must start `T::PAGES_PER_SLAB` live pages that nothing else
    /// points at.
    pub unsafe fn new(pages: NonNull<u8>) -> Self {
        debug_assert!(
            pages
                .addr()
                .get()
                .is_multiple_of(REGULAR_PAGE_ALIGNMENT.as_usize()),
            "slab pages are page-aligned"
        );
        Self(pages.cast())
    }

    /// The slot at `idx`.
    pub fn slot(&self, idx: usize) -> NonNull<PreAllocated<T>> {
        assert!(
            idx < T::OBJECT_PER_SLAB,
            "slot {idx} is outside the slab"
        );
        // SAFETY: `idx` is inside the slab, so the result stays in its
        // pages.
        unsafe { self.0.add(idx) }
    }

    /// The index of `object`, or `None` when it isn't a slot of this
    /// slab.
    pub fn index_of(&self, object: NonNull<T>) -> Option<NonMaxU16> {
        let offset =
            object.addr().get().checked_sub(self.0.addr().get())?;
        let slot_size = size_of::<PreAllocated<T>>();
        let idx = offset / slot_size;

        (offset.is_multiple_of(slot_size) && idx < T::OBJECT_PER_SLAB)
            .then(|| NonMaxU16::new(idx as u16))
            .flatten()
    }

    /// Links every slot to the one after it, and the last to none.
    ///
    /// # Safety
    ///
    /// No slot may be allocated.
    pub unsafe fn link_all(&mut self) {
        let mut slots =
            NonNull::slice_from_raw_parts(self.0, T::OBJECT_PER_SLAB);
        // SAFETY: `slots` covers this slab's pages, and the caller
        // guarantees no slot is allocated, so nothing else references
        // them.
        let slots = unsafe { slots.as_mut() };

        for (slot, next) in slots.iter_mut().zip(1..) {
            slot.next_free_idx = NonMaxU16::new(next);
        }
        if let Some(last) = slots.last_mut() {
            last.next_free_idx = None;
        }
    }
}
