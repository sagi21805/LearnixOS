use core::ptr::NonNull;

use nonmax::NonMaxU16;

use crate::{
    Generic64,
    descriptor::{
        FreeDetached, FreeHead, FullDetached, FullHead, Link, ListHead,
        PartialDetached, PartialHead, SlabDescriptor, SlabState, Unlink,
        Used, UsedSlab,
        meta::PartialMeta,
        partial::{Returned, Taken},
    },
    traits::Slab,
};
use typestate_groups::Morphic;

/// It is `repr(C)`, so its layout is the same for every `T`, which
/// [`SlabCaches::cache`] relies on.
#[derive(Debug)]
#[repr(C)]
pub struct SlabCache<T: Slab> {
    pub(crate) free: ListHead<T, FreeHead>,
    pub(crate) partial: ListHead<T, PartialHead>,
    pub(crate) full: ListHead<T, FullHead>,
}

// SAFETY: the descriptors the heads point at are reached only through
// this cache's lists, and only through `&mut self`, so moving the cache to
// another thread moves its lists with it.
unsafe impl<T: Slab> Send for SlabCache<T> {}

// The cache is three head pointers, whatever `T` is.
const _: () = assert!(size_of::<SlabCache<Generic64>>() == 24);

/// The slab caches, each slab type `T`'s at `T::SLAB_POSITION`, stored
/// erased so they fit in one array.
pub struct SlabCaches<const N: usize>([SlabCache<()>; N]);

impl<const N: usize> SlabCaches<N> {
    /// Every cache, empty.
    pub(crate) const fn new() -> Self {
        Self([const { SlabCache::new() }; N])
    }

    /// The cache of `T`.
    pub fn cache<T: Slab>(&mut self) -> &mut SlabCache<T> {
        let cache = NonNull::from_mut(&mut self.0[T::SLAB_POSITION]);
        // SAFETY: `Slab` is sealed, and `define_slab_system!` gives each
        // type its own position, so only `T` reaches this slot, and an
        // empty cache is valid for every `T`. `SlabCache` is `repr(C)`,
        // so `SlabCache<()>` and `SlabCache<T>` share a layout.
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

    /// Allocate a new slab descriptor, attaches it to the free slab list,
    /// and initialize it's page.
    ///
    /// TODO: THIS FUNCTION SHOULD MOVE TO THE SLAB ALLOCATOR AND GET A
    /// TYPE OF A SPECIFIC SLAB
    pub fn grow(&mut self) {
        // Allocate a new slab descriptor for this slab
        // let mut slab = unsafe {
        //     SLAB_ALLOCATOR.kmalloc::<SlabDescriptor<()>>().assign::<T>()
        // };

        // unsafe {
        //     *slab.as_mut() =
        //         SlabDescriptor::<T>::new(self.buddy_order, self.free)
        // }

        // self.take_ownership(slab);

        // self.free = Some(slab);
    }

    /// Takes a free object, and moves its slab to the list of its new
    /// state.
    pub(crate) fn take_object(&mut self) -> NonNull<T> {
        // SAFETY: this call holds no other reference to the partial head.
        if let Some(head) = unsafe { self.partial.first() } {
            let (allocation, final_state) = head.take_object();
            match final_state {
                Taken::Partial => {}
                Taken::Full => {
                    head.unlink(&mut self.partial)
                        .morph_mut::<FullDetached>()
                        .link(&mut self.full);
                }
            }
            return allocation;
        }

        // SAFETY: this call holds no other reference to the free head.
        if let Some(head) = unsafe { self.free.first() } {
            let partial =
                head.unlink(&mut self.free).morph_mut::<PartialDetached>();
            let (allocation, final_state) = partial.take_object();
            match final_state {
                Taken::Partial => {
                    partial.link(&mut self.partial);
                }
                Taken::Full => {
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
            // SAFETY: the caller guarantees `idx` is an allocated object
            // of `slab`.
            UsedSlab::PartialHead(partial) => unsafe {
                self.return_to_partial(idx, partial)
            },
            // SAFETY: as above.
            UsedSlab::Partial(partial) => unsafe {
                self.return_to_partial(idx, partial)
            },
            // SAFETY: as above.
            UsedSlab::FullHead(full) => unsafe {
                self.return_to_full(idx, full)
            },
            // SAFETY: as above.
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
        SlabDescriptor<T, S>: Unlink<
                T,
                Head = PartialHead,
                Detached = SlabDescriptor<T, PartialDetached>,
            >,
    {
        // SAFETY: the caller guarantees `idx` is allocated.
        match unsafe { partial.return_object(idx) } {
            Returned::Partial => {}
            Returned::Free => {
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
        SlabDescriptor<T, S>: Unlink<
                T,
                Head = FullHead,
                Detached = SlabDescriptor<T, FullDetached>,
            >,
    {
        let partial =
            full.unlink(&mut self.full).morph_mut::<PartialDetached>();
        // SAFETY: the caller guarantees `idx` is allocated.
        match unsafe { partial.return_object(idx) } {
            Returned::Partial => {
                partial.link(&mut self.partial);
            }
            Returned::Free => {
                partial.morph_mut::<FreeDetached>().link(&mut self.free);
            }
        }
    }
}
