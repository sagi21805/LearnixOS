use core::{hint::unreachable_unchecked, ptr::NonNull};

use nonmax::NonMaxU16;

use crate::{
    descriptor::{
        FreeDetached, FreeHead, Full, FullDetached, FullHead, Partial,
        PartialDetached, PartialHead, SlabStateKind, Used,
    },
    slab_address::SlabAddress,
    traits::Slab,
};
use typestate_groups::{Isomorphic, Morphic};

use super::descriptor::SlabDescriptor;

#[derive(Debug)]
pub struct SlabCache<T: Slab> {
    pub buddy_order: usize,
    pub free: Option<NonNull<SlabDescriptor<T, FreeHead>>>,
    pub partial: Option<NonNull<SlabDescriptor<T, PartialHead>>>,
    pub full: Option<NonNull<SlabDescriptor<T, FullHead>>>,
}

unsafe impl<T: Slab> Send for SlabCache<T> {}

const impl Default for SlabCache<()> {
    fn default() -> Self {
        SlabCache {
            buddy_order: 0,
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
    pub fn new(buddy_order: usize) -> SlabCache<T> {
        SlabCache {
            buddy_order,
            free: None,
            partial: None,
            full: None,
        }
    }

    /// Adds `full` to the full list.
    fn attach_or_set_full(
        &mut self,
        full: &mut SlabDescriptor<T, FullDetached>,
    ) {
        match self.full {
            // SAFETY: the head is live, and nothing else borrows it while
            // the cache is locked.
            Some(mut head) => unsafe { head.as_mut() }.link(full),
            None => {
                self.full = Some(NonNull::from_mut(
                    full.cast_state_mut::<FullHead>(),
                ))
            }
        }
    }

    /// Adds `free` to the free list.
    fn attach_or_set_free(
        &mut self,
        free: &mut SlabDescriptor<T, FreeDetached>,
    ) {
        match self.free {
            // SAFETY: the head is live, and nothing else borrows it while
            // the cache is locked.
            Some(mut head) => unsafe { head.as_mut() }.link(free),
            None => {
                self.free = Some(NonNull::from_mut(
                    free.cast_state_mut::<FreeHead>(),
                ))
            }
        }
    }

    /// Adds `partial` to the partial list.
    fn attach_or_set_partial(
        &mut self,
        partial: &mut SlabDescriptor<T, PartialDetached>,
    ) {
        match self.partial {
            Some(mut head) => {
                // SAFETY: the head is live, and nothing else borrows it
                // while the cache is locked.
                let head = unsafe { head.as_mut() };
                partial.next = head.next;
                head.next =
                    SlabAddress::from_non_null(NonNull::from_mut(partial));
            }
            None => {
                self.partial = Some(NonNull::from_mut(
                    partial.cast_state_mut::<PartialHead>(),
                ))
            }
        }
    }

    /// Removes the head of the free list.
    fn pop_free<'a>(
        &mut self,
    ) -> Option<&'a mut SlabDescriptor<T, FreeDetached>> {
        // SAFETY: the head is live, and nothing else borrows it while the
        // cache is locked.
        let head = unsafe { self.free?.as_mut() };
        // SAFETY: `next` links a live node of the free list.
        self.free = unsafe { head.next.as_non_null::<T, FreeHead>() };
        head.unlink();
        Some(head.cast_state_mut::<FreeDetached>())
    }

    /// Removes the head of the partial list.
    fn pop_partial<'a>(
        &mut self,
    ) -> &'a mut SlabDescriptor<T, PartialDetached> {
        let mut head = self.partial.expect("the partial list has a head");
        // SAFETY: the head is live, and nothing else borrows it while the
        // cache is locked.
        let head = unsafe { head.as_mut() };
        // SAFETY: `next` links a live node of the partial list.
        self.partial =
            unsafe { head.next.as_non_null::<T, PartialHead>() };
        head.next = SlabAddress::default();
        head.cast_state_mut::<PartialDetached>()
    }

    /// Unlinks `partial` from the partial list.
    fn detach_partial<'a>(
        &mut self,
        partial: &'a mut SlabDescriptor<T, Partial>,
    ) -> &'a mut SlabDescriptor<T, PartialDetached> {
        let target =
            SlabAddress::from_non_null(NonNull::from_mut(partial));
        let mut head =
            self.partial.expect("a partial slab is in the partial list");

        if SlabAddress::from_non_null(head) == target {
            // SAFETY: `next` links a live node of the partial list.
            self.partial =
                unsafe { partial.next.as_non_null::<T, PartialHead>() };
        } else {
            // SAFETY: the head is live and isn't `partial`, and nothing
            // else borrows it while the cache is locked.
            let mut link = &mut unsafe { head.as_mut() }.next;
            while *link != target {
                // SAFETY: `link` links a live node of the partial list
                // before `partial`, which nothing else borrows while the
                // cache is locked.
                let mut node = unsafe { link.as_non_null::<T, Partial>() }
                    .expect("`partial` is in the partial list");
                // SAFETY: as above.
                link = &mut unsafe { node.as_mut() }.next;
            }
            *link = partial.next;
        }

        partial.next = SlabAddress::default();
        partial.cast_state_mut::<PartialDetached>()
    }

    /// Unlinks `full` from the full list.
    fn detach_full<'a>(
        &mut self,
        full: &'a mut SlabDescriptor<T, Full>,
    ) -> &'a mut SlabDescriptor<T, FullDetached> {
        let target = SlabAddress::from_non_null(NonNull::from_mut(full));

        if self
            .full
            .is_some_and(|head| SlabAddress::from_non_null(head) == target)
        {
            // SAFETY: `next` links a live node of the full list.
            self.full = unsafe { full.next.as_non_null::<T, FullHead>() };
        }
        full.unlink();
        full.cast_state_mut::<FullDetached>()
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

    pub fn alloc(&mut self) -> NonNull<T> {
        if let Some(mut head) = self.partial {
            // SAFETY: the head is live, and nothing else borrows it while
            // the cache is locked.
            let (allocation, final_state) =
                unsafe { head.as_mut() }.alloc();
            match final_state {
                SlabStateKind::Partial => {}
                SlabStateKind::Full => {
                    let full =
                        self.pop_partial().morph_mut::<FullDetached>();
                    self.attach_or_set_full(full);
                }
                // SAFETY: allocating never frees a slab.
                SlabStateKind::Free => unsafe { unreachable_unchecked() },
            }
            return allocation;
        }

        if let Some(free) = self.pop_free() {
            let partial = free.morph_mut::<PartialDetached>();
            let (allocation, final_state) = partial.alloc();
            match final_state {
                SlabStateKind::Partial => {
                    self.attach_or_set_partial(partial)
                }
                SlabStateKind::Full => self.attach_or_set_full(
                    partial.morph_mut::<FullDetached>(),
                ),
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

    /// Frees object `idx` of `slab`, and moves `slab` to the list of its
    /// new state.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `slab`, and `slab` must be in
    /// one of this cache's lists.
    pub unsafe fn dealloc(
        &mut self,
        idx: NonMaxU16,
        slab: &mut SlabDescriptor<T, Used>,
    ) {
        // SAFETY: the caller's guarantees carry over.
        unsafe {
            match slab.is_partial_mut() {
                Ok(partial) => self.dealloc_partial(idx, partial),
                Err(full) => self.dealloc_full(idx, full),
            }
        }
    }

    /// Frees object `idx` of a slab in the partial list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `partial`.
    unsafe fn dealloc_partial(
        &mut self,
        idx: NonMaxU16,
        partial: &mut SlabDescriptor<T, Partial>,
    ) {
        // SAFETY: the caller guarantees `idx` is allocated.
        match unsafe { partial.dealloc(idx) } {
            SlabStateKind::Partial => {}
            SlabStateKind::Free => {
                let free = self
                    .detach_partial(partial)
                    .morph_mut::<FreeDetached>();
                self.attach_or_set_free(free);
            }
            // SAFETY: freeing never fills a slab.
            SlabStateKind::Full => unsafe { unreachable_unchecked() },
        }
    }

    /// Frees object `idx` of a slab in the full list.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of `full`.
    unsafe fn dealloc_full(
        &mut self,
        idx: NonMaxU16,
        full: &mut SlabDescriptor<T, Full>,
    ) {
        let partial =
            self.detach_full(full).morph_mut::<PartialDetached>();
        // SAFETY: the caller guarantees `idx` is allocated.
        match unsafe { partial.dealloc(idx) } {
            SlabStateKind::Partial => self.attach_or_set_partial(partial),
            SlabStateKind::Free => self
                .attach_or_set_free(partial.morph_mut::<FreeDetached>()),
            // SAFETY: freeing never fills a slab.
            SlabStateKind::Full => unsafe { unreachable_unchecked() },
        }
    }
}
