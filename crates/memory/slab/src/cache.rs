use core::ptr::NonNull;

use crate::{
    Generic64, SlabArena,
    descriptor::{
        FreeDetached, FreeHead, FullDetached, FullHead, InList, Link,
        ListHead, PartialDetached, PartialHead, SlabDescriptor, SlabState,
        Unlink, Used, UsedSlab,
        meta::PartialMeta,
        partial::{Allocated, Deallocated},
    },
    object_index::ObjectIndex,
    traits::{Slab, Unknown},
};
use typestate_groups::Morphic;

#[repr(C)]
#[derive(Debug)]
pub struct SlabCache<T: Slab> {
    pub(crate) free: ListHead<T, FreeHead>,
    pub(crate) partial: ListHead<T, PartialHead>,
    pub(crate) full: ListHead<T, FullHead>,
}

// SAFETY: the lists are reached only through `&mut self`, so they move
// with the cache.
unsafe impl<T: Slab> Send for SlabCache<T> {}

// The cache is three head indices, whatever `T` is.
const _: () = assert!(size_of::<SlabCache<Generic64>>() == 12);

/// The slab caches, each slab type `T`'s at `T::SLAB_POSITION`, stored as
/// [`Unknown`] caches so they fit in one array.
pub struct SlabCaches<const N: usize>([SlabCache<Unknown>; N]);

impl<const N: usize> SlabCaches<N> {
    /// Every cache, empty.
    pub(crate) const fn new() -> Self {
        Self([const { SlabCache::new() }; N])
    }

    /// The cache of `T`.
    pub fn cache<T: Slab>(&mut self) -> &mut SlabCache<T> {
        let cache = NonNull::from_mut(&mut self.0[T::SLAB_POSITION]);
        // SAFETY: the `Slab` contract gives each type its own position, so
        // only `T` reaches this slot. `repr(C)` keeps the layouts equal.
        unsafe { cache.cast::<SlabCache<T>>().as_mut() }
    }
}

impl<T: Slab> SlabCache<T> {
    /// An empty cache.
    pub(crate) const fn new() -> SlabCache<T> {
        SlabCache {
            free: ListHead::empty(),
            partial: ListHead::empty(),
            full: ListHead::empty(),
        }
    }

    /// Allocates an object from a partial slab, or from a free one when
    /// no slab is partial, and moves its slab to the list of its new
    /// state.
    ///
    /// Returns `None` when both lists are empty, so the cache needs a new
    /// slab.
    pub(crate) fn try_allocate(
        &mut self,
        arena: &impl SlabArena,
    ) -> Option<NonNull<T>> {
        self.allocate_from_partial(arena)
            .or_else(|| self.allocate_from_free(arena))
    }

    /// Allocates an object from the head of the partial list, and moves
    /// the slab to the full list when it was its last free one.
    ///
    /// Returns `None` when the partial list is empty.
    fn allocate_from_partial(
        &mut self,
        arena: &impl SlabArena,
    ) -> Option<NonNull<T>> {
        // SAFETY: `&mut self` is the only path to the cache's lists, so
        // nothing else references the head or frees its page meanwhile.
        let head = unsafe { self.partial.first(arena) }?;
        let (object, allocated) = head.allocate_slot(arena);
        match allocated {
            Allocated::Partial => {}
            Allocated::Full => {
                head.unlink(arena, &mut self.partial)
                    .morph_mut::<FullDetached>()
                    .link(arena, &mut self.full);
            }
        }
        Some(object)
    }

    /// Allocates an object from the head of the free list and attaches it
    /// to the partial list.
    ///
    /// Returns `None` when the free list is empty.
    fn allocate_from_free(
        &mut self,
        arena: &impl SlabArena,
    ) -> Option<NonNull<T>> {
        // SAFETY: `&mut self` is the only path to the cache's lists, so
        // nothing else references the head or frees its page meanwhile.
        let head = unsafe { self.free.first(arena) }?;
        let free = head.unlink(arena, &mut self.free);
        Some(self.allocate_and_link(arena, free))
    }

    /// Links the slots of `free`, a free slab in no list such as a new
    /// one, allocates an object from it, and links it into the partial
    /// list, or into the full list when it holds one object.
    pub(crate) fn allocate_and_link(
        &mut self,
        arena: &impl SlabArena,
        free: &mut SlabDescriptor<T, FreeDetached>,
    ) -> NonNull<T> {
        // SAFETY: a free slab has no allocated slot.
        unsafe { arena.objects_of(free).link_all() };
        let partial = free.morph_mut::<PartialDetached>();
        let (object, allocated) = partial.allocate_slot(arena);
        match allocated {
            Allocated::Partial => partial.link(arena, &mut self.partial),
            Allocated::Full => partial
                .morph_mut::<FullDetached>()
                .link(arena, &mut self.full),
        }
        object
    }

    /// Deallocates object `idx` of `slab`, and moves `slab` to the list of
    /// its new state.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `slab`, and `slab` must be in
    /// one of this cache's lists.
    pub(crate) unsafe fn deallocate_object(
        &mut self,
        arena: &impl SlabArena,
        idx: ObjectIndex,
        slab: &mut SlabDescriptor<T, Used>,
    ) {
        match slab.classify_mut(arena) {
            UsedSlab::PartialHead(partial) => unsafe {
                self.deallocate_from_partial(arena, idx, partial)
            },
            UsedSlab::Partial(partial) => unsafe {
                self.deallocate_from_partial(arena, idx, partial)
            },
            UsedSlab::FullHead(full) => unsafe {
                self.deallocate_from_full(arena, idx, full)
            },
            UsedSlab::Full(full) => unsafe {
                self.deallocate_from_full(arena, idx, full)
            },
        }
    }

    /// Deallocates object `idx` of a slab in the partial list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `partial`.
    unsafe fn deallocate_from_partial<S>(
        &mut self,
        arena: &impl SlabArena,
        idx: ObjectIndex,
        partial: &mut SlabDescriptor<T, S>,
    ) where
        S: SlabState<Meta = PartialMeta>,
        SlabDescriptor<T, S>: InList<T, PartialHead>,
    {
        match unsafe { partial.deallocate_slot(arena, idx) } {
            Deallocated::Partial => {}
            Deallocated::Free => {
                partial
                    .unlink(arena, &mut self.partial)
                    .morph_mut::<FreeDetached>()
                    .link(arena, &mut self.free);
            }
        }
    }

    /// Deallocates object `idx` of a slab in the full list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `full`.
    unsafe fn deallocate_from_full<S>(
        &mut self,
        arena: &impl SlabArena,
        idx: ObjectIndex,
        full: &mut SlabDescriptor<T, S>,
    ) where
        S: SlabState,
        SlabDescriptor<T, S>: InList<T, FullHead>,
    {
        let partial = full
            .unlink(arena, &mut self.full)
            .morph_mut::<PartialDetached>();
        match unsafe { partial.deallocate_slot(arena, idx) } {
            Deallocated::Partial => {
                partial.link(arena, &mut self.partial);
            }
            Deallocated::Free => {
                partial
                    .morph_mut::<FreeDetached>()
                    .link(arena, &mut self.free);
            }
        }
    }
}
