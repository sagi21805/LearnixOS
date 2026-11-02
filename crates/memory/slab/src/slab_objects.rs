use core::ptr::NonNull;

use common::address_types::VirtualAddress;

use crate::{
    object_index::ObjectIndex, preallocated::PreAllocated, traits::Slab,
};

/// The object slots of one slab, which
/// [`SlabArena::objects_of`](crate::SlabArena::objects_of) rebuilds from
/// the slab's descriptor.
///
/// Many values may point at the same slots. Whoever holds the slab's
/// descriptor by `&mut` is its one manager, so only that code writes the
/// free slots.
#[repr(transparent)]
pub struct SlabObjects<T: Slab>(NonNull<T::PreAllocatedArray>);

impl<T: Slab> SlabObjects<T> {
    /// The slots on the `T::PAGES_PER_SLAB` pages at `pages`.
    ///
    /// # Safety
    ///
    /// `pages` must start the `T::PAGES_PER_SLAB` live pages of one slab.
    pub fn new(pages: NonNull<u8>) -> Self {
        debug_assert!(
            pages.is_aligned_to(T::PSIZE.alignment().as_usize()),
            "slab pages are page-aligned"
        );
        Self(pages.cast())
    }

    /// Where the slab's pages start.
    pub fn address(&self) -> VirtualAddress {
        VirtualAddress::from(self.0.addr().get())
    }

    /// The object at `idx`.
    ///
    /// # Safety
    ///
    /// `idx` must be less than `T::OBJECTS_PER_SLAB`.
    pub unsafe fn object_at(
        &self,
        idx: ObjectIndex,
    ) -> NonNull<PreAllocated<T>> {
        // SAFETY: the caller keeps `idx` inside the slab.
        unsafe { self.objects().get_unchecked_mut(idx.get() as usize) }
    }

    /// The index of `object`, or `None` when it isn't an object of this
    /// slab.
    pub fn index_of(&self, object: NonNull<T>) -> Option<ObjectIndex> {
        let offset = object.addr().get().wrapping_sub(self.0.addr().get());
        T::object_index_at(offset)
    }

    /// Links every slot to the one after it, and the last to none.
    ///
    /// # Safety
    ///
    /// No slot may be allocated.
    pub unsafe fn link_all(&mut self) {
        // SAFETY: no slot is allocated, so nothing else references them.
        let slots = unsafe { self.objects().as_mut() };

        for (slot, next) in slots.iter_mut().zip(1..) {
            slot.next_free_idx = ObjectIndex::new(next);
        }
        if let Some(last) = slots.last_mut() {
            last.next_free_idx = None;
        }
    }

    /// Every object of the slab, as a slice pointer whose length comes
    /// from `T::PreAllocatedArray`.
    fn objects(&self) -> NonNull<[PreAllocated<T>]> { self.0 }
}
