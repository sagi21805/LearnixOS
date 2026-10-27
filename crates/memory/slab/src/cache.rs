use core::ptr::NonNull;

use nonmax::NonMaxU16;

use crate::{
    Generic64,
    descriptor::{
        FreeDetached, FreeHead, FullDetached, FullHead, InList, Link,
        ListHead, PartialDetached, PartialHead, SlabDescriptor, SlabState,
        Unlink, Used, UsedSlab,
        meta::PartialMeta,
        partial::{Allocated, Deallocated},
    },
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

// The cache is three head pointers, whatever `T` is.
const _: () = assert!(size_of::<SlabCache<Generic64>>() == 24);

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
        // SAFETY: `Slab` is sealed and each type has its own position, so
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
    pub(crate) fn try_allocate(&mut self) -> Option<NonNull<T>> {
        self.allocate_from_partial()
            .or_else(|| self.allocate_from_free())
    }

    /// Allocates an object from the head of the partial list, and moves
    /// the slab to the full list when it was its last free one.
    ///
    /// Returns `None` when the partial list is empty.
    fn allocate_from_partial(&mut self) -> Option<NonNull<T>> {
        // SAFETY: `&mut self` is the only path to the cache's lists, so
        // nothing else references the head or frees its page meanwhile.
        let head = unsafe { self.partial.first() }?;
        let (object, allocated) = head.allocate_slot();
        match allocated {
            Allocated::Partial => {}
            Allocated::Full => {
                head.unlink(&mut self.partial)
                    .morph_mut::<FullDetached>()
                    .link(&mut self.full);
            }
        }
        Some(object)
    }

    /// Allocates an object from the head of the free list and attaches it
    /// to the partial list.
    ///
    /// Returns `None` when the free list is empty.
    fn allocate_from_free(&mut self) -> Option<NonNull<T>> {
        // SAFETY: `&mut self` is the only path to the cache's lists, so
        // nothing else references the head or frees its page meanwhile.
        let head = unsafe { self.free.first() }?;
        let free = head.unlink(&mut self.free);
        Some(self.allocate_and_link(free))
    }

    /// Allocates an object from `free`, a free slab in no list, such as a
    /// new one, and links it into the partial list, or into the full
    /// list when it holds one object.
    pub(crate) fn allocate_and_link(
        &mut self,
        free: &mut SlabDescriptor<T, FreeDetached>,
    ) -> NonNull<T> {
        let partial = free.morph_mut::<PartialDetached>();
        let (object, allocated) = partial.allocate_slot();
        match allocated {
            Allocated::Partial => partial.link(&mut self.partial),
            Allocated::Full => {
                partial.morph_mut::<FullDetached>().link(&mut self.full)
            }
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
        idx: NonMaxU16,
        slab: &mut SlabDescriptor<T, Used>,
    ) {
        match slab.classify_mut() {
            UsedSlab::PartialHead(partial) => unsafe {
                self.deallocate_from_partial(idx, partial)
            },
            UsedSlab::Partial(partial) => unsafe {
                self.deallocate_from_partial(idx, partial)
            },
            UsedSlab::FullHead(full) => unsafe {
                self.deallocate_from_full(idx, full)
            },
            UsedSlab::Full(full) => unsafe {
                self.deallocate_from_full(idx, full)
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
        idx: NonMaxU16,
        partial: &mut SlabDescriptor<T, S>,
    ) where
        S: SlabState<Meta = PartialMeta>,
        SlabDescriptor<T, S>: InList<T, PartialHead>,
    {
        match unsafe { partial.deallocate_slot(idx) } {
            Deallocated::Partial => {}
            Deallocated::Free => {
                partial
                    .unlink(&mut self.partial)
                    .morph_mut::<FreeDetached>()
                    .link(&mut self.free);
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
        idx: NonMaxU16,
        full: &mut SlabDescriptor<T, S>,
    ) where
        S: SlabState,
        SlabDescriptor<T, S>: InList<T, FullHead>,
    {
        let partial =
            full.unlink(&mut self.full).morph_mut::<PartialDetached>();
        match unsafe { partial.deallocate_slot(idx) } {
            Deallocated::Partial => {
                partial.link(&mut self.partial);
            }
            Deallocated::Free => {
                partial.morph_mut::<FreeDetached>().link(&mut self.free);
            }
        }
    }
}
