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

#[derive(Debug)]
pub struct SlabCache<T: Slab> {
    pub(crate) free: ListHead<T, FreeHead>,
    pub(crate) partial: ListHead<T, PartialHead>,
    pub(crate) full: ListHead<T, FullHead>,
}

unsafe impl<T: Slab> Send for SlabCache<T> {}

// The cache is three head pointers, whatever `T` is.
const _: () = assert!(size_of::<SlabCache<Generic64>>() == 24);

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
        // SAFETY: the caller's guarantees carry over.
        unsafe {
            match slab.classify_mut() {
                UsedSlab::PartialHead(partial) => {
                    self.return_to_partial(idx, partial)
                }
                UsedSlab::Partial(partial) => {
                    self.return_to_partial(idx, partial)
                }
                UsedSlab::FullHead(full) => self.return_to_full(idx, full),
                UsedSlab::Full(full) => self.return_to_full(idx, full),
            }
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
