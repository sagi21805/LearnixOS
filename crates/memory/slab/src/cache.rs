use core::ptr::NonNull;

use nonmax::NonMaxU16;

use crate::{
    Generic64,
    descriptor::{
        FreeDetached, FreeHead, FullDetached, FullHead, InList, Link,
        ListHead, PartialDetached, PartialHead, SlabDescriptor, SlabState,
        Unlink, Used, UsedSlab,
        meta::PartialMeta,
        partial::{Claimed, Released},
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

    /// Takes a free object, and moves its slab to the list of its new
    /// state.
    pub(crate) fn take_object(&mut self) -> NonNull<T> {
        if let Some(head) = unsafe { self.partial.first() } {
            let (allocation, final_state) = head.claim_slot();
            match final_state {
                Claimed::Partial => {}
                Claimed::Full => {
                    head.unlink(&mut self.partial)
                        .morph_mut::<FullDetached>()
                        .link(&mut self.full);
                }
            }
            return allocation;
        }

        if let Some(head) = unsafe { self.free.first() } {
            let partial =
                head.unlink(&mut self.free).morph_mut::<PartialDetached>();
            let (allocation, final_state) = partial.claim_slot();
            match final_state {
                Claimed::Partial => {
                    partial.link(&mut self.partial);
                }
                Claimed::Full => {
                    partial
                        .morph_mut::<FullDetached>()
                        .link(&mut self.full);
                }
            }
            return allocation;
        }

        todo!(
            "Handle cases where partial and free are full, and \
             allocation from the page allocator is needed."
        )
    }

    /// Returns object `idx` of `slab`, and moves `slab` to the list of
    /// its new state.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `slab`, and `slab` must be in
    /// one of this cache's lists.
    pub(crate) unsafe fn return_object(
        &mut self,
        idx: NonMaxU16,
        slab: &mut SlabDescriptor<T, Used>,
    ) {
        match slab.classify_mut() {
            UsedSlab::PartialHead(partial) => unsafe {
                self.return_to_partial(idx, partial)
            },
            UsedSlab::Partial(partial) => unsafe {
                self.return_to_partial(idx, partial)
            },
            UsedSlab::FullHead(full) => unsafe {
                self.return_to_full(idx, full)
            },
            UsedSlab::Full(full) => unsafe {
                self.return_to_full(idx, full)
            },
        }
    }

    /// Returns object `idx` of a slab in the partial list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `partial`.
    unsafe fn return_to_partial<S>(
        &mut self,
        idx: NonMaxU16,
        partial: &mut SlabDescriptor<T, S>,
    ) where
        S: SlabState<Meta = PartialMeta>,
        SlabDescriptor<T, S>: InList<T, PartialHead>,
    {
        match unsafe { partial.release_slot(idx) } {
            Released::Partial => {}
            Released::Free => {
                partial
                    .unlink(&mut self.partial)
                    .morph_mut::<FreeDetached>()
                    .link(&mut self.free);
            }
        }
    }

    /// Returns object `idx` of a slab in the full list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `full`.
    unsafe fn return_to_full<S>(
        &mut self,
        idx: NonMaxU16,
        full: &mut SlabDescriptor<T, S>,
    ) where
        S: SlabState,
        SlabDescriptor<T, S>: InList<T, FullHead>,
    {
        let partial =
            full.unlink(&mut self.full).morph_mut::<PartialDetached>();
        match unsafe { partial.release_slot(idx) } {
            Released::Partial => {
                partial.link(&mut self.partial);
            }
            Released::Free => {
                partial.morph_mut::<FreeDetached>().link(&mut self.free);
            }
        }
    }
}
