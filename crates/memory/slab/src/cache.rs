use core::{hint::unreachable_unchecked, ptr::NonNull};

use nonmax::NonMaxU16;

use crate::{
    descriptor::{
        FreeDetached, FreeHead, Full, FullDetached, FullHead, ListNode,
        Partial, PartialDetached, PartialHead, SlabStateKind, Used,
    },
    slab_address::SlabAddress,
    traits::Slab,
};
use typestate_groups::Morphic;

use super::descriptor::SlabDescriptor;

#[derive(Debug)]
pub struct SlabCache<T: Slab> {
    pub free: Option<NonNull<SlabDescriptor<T, FreeHead>>>,
    pub partial: Option<NonNull<SlabDescriptor<T, PartialHead>>>,
    pub full: Option<NonNull<SlabDescriptor<T, FullHead>>>,
}

unsafe impl<T: Slab> Send for SlabCache<T> {}

const impl Default for SlabCache<()> {
    fn default() -> Self {
        SlabCache {
            free: None,
            partial: None,
            full: None,
        }
    }
}

impl SlabCache<()> {
    pub unsafe fn assign<T: Slab>(&mut self) -> &mut SlabCache<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Slab> SlabCache<T> {
    pub fn new() -> SlabCache<T> {
        SlabCache {
            free: None,
            partial: None,
            full: None,
        }
    }

    /// Removes the head of the free list.
    ///
    /// # Safety
    ///
    /// The returned descriptor must not outlive its page, and no other
    /// reference to it may be used while this one is.
    unsafe fn pop_free<'a>(
        &mut self,
    ) -> Option<&'a mut SlabDescriptor<T, FreeDetached>> {
        // SAFETY: the head is live, and nothing else borrows it while the
        // cache is locked.
        let head = unsafe { self.free?.as_mut() };
        let free = head.unlink(&mut self.free);

        let address = SlabAddress::from_non_null(NonNull::from_mut(free));
        debug_assert!(
            free.next == SlabAddress::default()
                && free.state.get_prev() == SlabAddress::default(),
            "a popped slab is unlinked"
        );
        debug_assert!(
            self.free.map(SlabAddress::from_non_null) != Some(address),
            "a popped slab isn't the head"
        );
        Some(free)
    }

    /// Removes the head of the partial list.
    ///
    /// # Safety
    ///
    /// The returned descriptor must not outlive its page, and no other
    /// reference to it may be used while this one is.
    unsafe fn pop_partial<'a>(
        &mut self,
    ) -> &'a mut SlabDescriptor<T, PartialDetached> {
        let mut head = self.partial.expect("the partial list has a head");
        // SAFETY: the head is live, and nothing else borrows it while the
        // cache is locked.
        let partial = unsafe { head.as_mut() }.unlink(&mut self.partial);

        debug_assert!(
            self.partial.map(SlabAddress::from_non_null)
                != Some(SlabAddress::from_non_null(NonNull::from_mut(
                    partial
                ))),
            "a popped slab isn't the head"
        );
        partial
    }

    pub unsafe fn as_unit(self) -> SlabCache<()> {
        unsafe { core::mem::transmute(self) }
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
        if let Some(mut head) = self.partial {
            // SAFETY: the head is live, and nothing else borrows it while
            // the cache is locked.
            let (allocation, final_state) =
                unsafe { head.as_mut() }.take_object();
            match final_state {
                SlabStateKind::Partial => {}
                SlabStateKind::Full => {
                    // SAFETY: the slab is used only inside this call,
                    // while the cache is locked.
                    let full = unsafe { self.pop_partial() }
                        .morph_mut::<FullDetached>();
                    full.link(&mut self.full);
                }
                // SAFETY: allocating never frees a slab.
                SlabStateKind::Free => unsafe { unreachable_unchecked() },
            }
            return allocation;
        }

        // SAFETY: the slab is used only inside this call, while the
        // cache is locked.
        if let Some(free) = unsafe { self.pop_free() } {
            let partial = free.morph_mut::<PartialDetached>();
            let (allocation, final_state) = partial.take_object();
            match final_state {
                SlabStateKind::Partial => {
                    partial.link(&mut self.partial);
                }
                SlabStateKind::Full => {
                    partial
                        .morph_mut::<FullDetached>()
                        .link(&mut self.full);
                }
                // SAFETY: allocating never frees a slab.
                SlabStateKind::Free => unsafe { unreachable_unchecked() },
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
            match slab.is_partial_mut() {
                Ok(partial) => self.return_to_partial(idx, partial),
                Err(full) => self.return_to_full(idx, full),
            }
        }
    }

    /// Returns object `idx` of a slab in the partial list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `partial`.
    unsafe fn return_to_partial(
        &mut self,
        idx: NonMaxU16,
        partial: &mut SlabDescriptor<T, Partial>,
    ) {
        // SAFETY: the caller guarantees `idx` is allocated.
        match unsafe { partial.return_object(idx) } {
            SlabStateKind::Partial => {}
            SlabStateKind::Free => {
                partial
                    .unlink(&mut self.partial)
                    .morph_mut::<FreeDetached>()
                    .link(&mut self.free);
            }
            // SAFETY: freeing never fills a slab.
            SlabStateKind::Full => unsafe { unreachable_unchecked() },
        }
    }

    /// Returns object `idx` of a slab in the full list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `full`.
    unsafe fn return_to_full(
        &mut self,
        idx: NonMaxU16,
        full: &mut SlabDescriptor<T, Full>,
    ) {
        let partial =
            full.unlink(&mut self.full).morph_mut::<PartialDetached>();
        // SAFETY: the caller guarantees `idx` is allocated.
        match unsafe { partial.return_object(idx) } {
            SlabStateKind::Partial => {
                partial.link(&mut self.partial);
            }
            SlabStateKind::Free => {
                partial.morph_mut::<FreeDetached>().link(&mut self.free);
            }
            // SAFETY: freeing never fills a slab.
            SlabStateKind::Full => unsafe { unreachable_unchecked() },
        }
    }
}
