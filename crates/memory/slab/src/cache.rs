use core::{hint::unreachable_unchecked, ptr::NonNull};

use nonmax::NonMaxU16;

use crate::{
    descriptor::{
        FreeDetached, FreeHead, Full, FullDetached, FullHead, Partial,
        PartialDetached, PartialHead, SlabState, SlabStateKind, Used,
        meta::FullFreeMeta,
    },
    slab_address::SlabAddress,
    traits::Slab,
};
use extend::ext;
use typestate_groups::{ByMut, CastableState, Isomorphic, Morphic};

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
        let free = self.free.remove::<FreeDetached, _>(head);

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
        let head = unsafe { head.as_mut() };
        // SAFETY: `next` links a live node of the partial list.
        self.partial =
            unsafe { head.next.as_non_null::<T, PartialHead>() };
        head.next = SlabAddress::default();

        debug_assert!(
            self.partial.map(SlabAddress::from_non_null)
                != Some(SlabAddress::from_non_null(NonNull::from_mut(
                    head
                ))),
            "a popped slab isn't the head"
        );
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
                    // SAFETY: the slab is used only inside this call,
                    // while the cache is locked.
                    let full = unsafe { self.pop_partial() }
                        .morph_mut::<FullDetached>();
                    self.full.push(full);
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
            let (allocation, final_state) = partial.alloc();
            match final_state {
                SlabStateKind::Partial => {
                    self.attach_or_set_partial(partial)
                }
                SlabStateKind::Full => {
                    self.full.push(partial.morph_mut::<FullDetached>())
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
                self.free.push(free);
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
        let partial = self
            .full
            .remove::<FullDetached, _>(full)
            .morph_mut::<PartialDetached>();
        // SAFETY: the caller guarantees `idx` is allocated.
        match unsafe { partial.dealloc(idx) } {
            SlabStateKind::Partial => self.attach_or_set_partial(partial),
            SlabStateKind::Free => {
                self.free.push(partial.morph_mut::<FreeDetached>())
            }
            // SAFETY: freeing never fills a slab.
            SlabStateKind::Full => unsafe { unreachable_unchecked() },
        }
    }
}

/// The head of the doubly-linked free or full list.
#[ext]
impl<T: Slab, H: SlabState<Meta = FullFreeMeta>>
    Option<NonNull<SlabDescriptor<T, H>>>
{
    /// Adds `node` to the list, as its head when the list is empty.
    fn push<D: SlabState<Meta = FullFreeMeta>>(
        &mut self,
        node: &mut SlabDescriptor<T, D>,
    ) where
        SlabDescriptor<T, D>:
            CastableState<H, ByMut, Target = SlabDescriptor<T, H>>,
    {
        match self {
            // SAFETY: the head is live, and nothing else borrows it while
            // the cache is locked.
            Some(head) => unsafe { head.as_mut() }.link(node),
            None => {
                *self = Some(NonNull::from_mut(node.cast_state_mut::<H>()))
            }
        }
    }

    /// Unlinks `node` from the list, moving the head on when `node` is
    /// it.
    fn remove<'a, D: SlabState, A: SlabState<Meta = FullFreeMeta>>(
        &mut self,
        node: &'a mut SlabDescriptor<T, A>,
    ) -> &'a mut SlabDescriptor<T, D>
    where
        SlabDescriptor<T, A>:
            CastableState<D, ByMut, Target = SlabDescriptor<T, D>>,
    {
        let address = SlabAddress::from_non_null(NonNull::from_mut(node));
        if self.is_some_and(|head| {
            SlabAddress::from_non_null(head) == address
        }) {
            // SAFETY: `next` links a live node of this list.
            *self = unsafe { node.next.as_non_null::<T, H>() };
        }
        node.unlink();
        node.cast_state_mut::<D>()
    }
}
