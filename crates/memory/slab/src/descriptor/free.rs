use core::marker::PhantomData;

use crate::{
    descriptor::{
        FreeDetached, ListState, PartialDetached, SlabDescriptor,
        meta::FullFreeMeta,
    },
    slab_address::SlabAddress,
    traits::Slab,
};
use typestate_groups::MorphFrom;

impl<T: Slab> SlabDescriptor<T, FreeDetached> {
    /// A free slab in no list.
    ///
    /// Its slots aren't linked into a free list until
    /// [`SlabCache::allocate_and_link`](crate::cache::SlabCache::allocate_and_link)
    /// makes it partial.
    pub fn new() -> Self {
        SlabDescriptor {
            state: FullFreeMeta::default(),
            next: SlabAddress::default(),
            _object: PhantomData,
        }
    }
}

impl<T: Slab> Default for SlabDescriptor<T, FreeDetached> {
    fn default() -> Self { Self::new() }
}

/// A partial slab whose last object was freed becomes free.
impl<T: Slab> MorphFrom<SlabDescriptor<T, PartialDetached>>
    for SlabDescriptor<T, FreeDetached>
{
    fn morph_from(src: SlabDescriptor<T, PartialDetached>) -> Self {
        debug_assert_eq!(src.state.get_total_allocated(), 0);
        Self::new()
    }
}

/// A slab leaving the free list, without its previous or next node.
impl<T: Slab, S> MorphFrom<SlabDescriptor<T, S>>
    for SlabDescriptor<T, FreeDetached>
where
    S: ListState<Detached = FreeDetached>,
{
    fn morph_from(_src: SlabDescriptor<T, S>) -> Self { Self::new() }
}
