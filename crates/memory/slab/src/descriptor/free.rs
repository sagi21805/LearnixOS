extern crate alloc;

use crate::{
    descriptor::{
        FreeDetached, PartialDetached, SlabDescriptor, meta::FullFreeMeta,
    },
    preallocated::PreAllocated,
    slab_address::SlabAddress,
    traits::Slab,
};
use alloc::alloc::{Layout, alloc};
use common::constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE};
use core::ptr::NonNull;
use nonmax::NonMaxU16;
use typestate_groups::MorphFrom;

impl<T: Slab> SlabDescriptor<T, FreeDetached> {
    /// Create a new, free slab descriptor.
    #[expect(
        clippy::new_without_default,
        reason = "`new` allocates the slab's pages, which `Default` \
                  shouldn't hide"
    )]
    pub fn new() -> SlabDescriptor<T, FreeDetached> {
        todo!("Use direct call on buddy allocator for certain page size.");
        // SAFETY: the layout has a non-zero size and a power-of-two
        // alignment.
        let objects = unsafe {
            NonNull::new_unchecked(alloc(
                Layout::from_size_alignment_unchecked(
                    T::PAGES_PER_SLAB * REGULAR_PAGE_SIZE,
                    REGULAR_PAGE_ALIGNMENT,
                ),
            ))
            .cast::<PreAllocated<T>>()
        };

        let mut free = SlabDescriptor {
            state: FullFreeMeta::default(),
            objects,
            next: SlabAddress::default(),
        };
        free.link_objects();
        free
    }
}

/// A partial slab whose last object was freed becomes free, with its
/// free list starting at index 0 again.
impl<T: Slab> MorphFrom<SlabDescriptor<T, PartialDetached>>
    for SlabDescriptor<T, FreeDetached>
{
    fn morph_from(src: SlabDescriptor<T, PartialDetached>) -> Self {
        debug_assert_eq!(src.state.get_total_allocated(), 0);

        let mut free = SlabDescriptor {
            state: FullFreeMeta::default(),
            objects: src.objects,
            next: SlabAddress::default(),
        };
        free.link_objects();
        free
    }
}

impl<T: Slab> SlabDescriptor<T, FreeDetached> {
    /// Links each object to the one after it, and the last to none.
    fn link_objects(&mut self) {
        let mut objects = NonNull::slice_from_raw_parts(
            self.objects,
            T::OBJECT_PER_SLAB,
        );
        // SAFETY: `objects` covers this slab's objects, which are all
        // free, so nothing else references them.
        let objects = unsafe { objects.as_mut() };

        for (object, next) in objects.iter_mut().zip(1..) {
            object.next_free_idx = NonMaxU16::new(next);
        }
        if let Some(last) = objects.last_mut() {
            last.next_free_idx = None;
        }
    }
}
