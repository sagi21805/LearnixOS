use crate::{
    descriptor::{
        FreeDetached, ListState, PartialDetached, SlabDescriptor,
        meta::FullFreeMeta,
    },
    slab_address::SlabAddress,
    slab_objects::SlabObjects,
    traits::Slab,
};
use typestate_groups::MorphFrom;

impl<T: Slab> SlabDescriptor<T, FreeDetached> {
    /// A free slab over `objects`, in no list, with every slot on its
    /// free list.
    ///
    /// # Safety
    ///
    /// No slot of `objects` may be allocated.
    pub unsafe fn new(mut objects: SlabObjects<T>) -> Self {
        // SAFETY: the caller guarantees no slot is allocated.
        unsafe { objects.link_all() };
        SlabDescriptor {
            state: FullFreeMeta::default(),
            objects,
            next: SlabAddress::default(),
        }
    }
}

/// A partial slab whose last object was freed becomes free, with its
/// free list starting at index 0 again.
impl<T: Slab> MorphFrom<SlabDescriptor<T, PartialDetached>>
    for SlabDescriptor<T, FreeDetached>
{
    fn morph_from(src: SlabDescriptor<T, PartialDetached>) -> Self {
        debug_assert_eq!(src.state.get_total_allocated(), 0);
        // SAFETY: the slab's last object was freed.
        unsafe { Self::new(src.objects) }
    }
}

/// A slab leaving the free list keeps its slots, without its previous
/// or next node.
impl<T: Slab, S> MorphFrom<SlabDescriptor<T, S>>
    for SlabDescriptor<T, FreeDetached>
where
    S: ListState<Detached = FreeDetached>,
{
    fn morph_from(src: SlabDescriptor<T, S>) -> Self {
        SlabDescriptor {
            state: FullFreeMeta::default(),
            objects: src.objects,
            next: SlabAddress::default(),
        }
    }
}
