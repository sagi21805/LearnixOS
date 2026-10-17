extern crate alloc;

use crate::{
    descriptor::{
        FreeDetached, PartialDetached, SlabDescriptor, meta::FullFreeMeta,
    },
    slab_address::SlabAddress,
    slab_objects::SlabObjects,
    traits::Slab,
};
use alloc::alloc::alloc;
use core::ptr::NonNull;
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
        // SAFETY: a slab's layout has at least one page.
        let pages = unsafe { alloc(T::SLAB_LAYOUT) };
        let pages = NonNull::new(pages)
            .expect("TODO: handle running out of pages");
        // SAFETY: the pages were just allocated, so nothing else points at
        // them.
        let objects = unsafe { SlabObjects::new(pages) };

        let mut free = SlabDescriptor {
            state: FullFreeMeta::default(),
            objects,
            next: SlabAddress::default(),
        };
        // SAFETY: a free slab has no allocated slot.
        unsafe { free.objects.link_all() };
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
        // SAFETY: a free slab has no allocated slot.
        unsafe { free.objects.link_all() };
        free
    }
}
