use core::{hint::unreachable_unchecked, ptr::NonNull};

use nonmax::NonMaxU16;

use crate::{
    descriptor::{
        Attach, ErasedLink, FreeDetached, FreeHead, FullDetached,
        FullHead, Partial, PartialDetached, PartialHead, SlabStateKind,
        Used,
    },
    slab_address::SlabAddress,
    traits::Slab,
};
use typestate_groups::Morphic;

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

    fn attach_or_set_full(
        &mut self,
        other: &mut SlabDescriptor<T, FullDetached>,
    ) {
        match self.full.map(|mut p| unsafe { p.as_mut() }) {
            Some(full) => {
                full.attach(other);
            }
            None => {
                self.full = Some(NonNull::from_mut(other.as_head()));
            }
        };
    }

    fn attach_or_set_free(
        &mut self,
        other: &mut SlabDescriptor<T, FreeDetached>,
    ) {
        match self.free.map(|mut p| unsafe { p.as_mut() }) {
            Some(free) => {
                free.attach(other);
            }
            None => {
                self.free = Some(NonNull::from_mut(other.as_head()));
            }
        };
    }

    fn attach_or_set_partial(
        &mut self,
        other: &mut SlabDescriptor<T, PartialDetached>,
    ) {
        match self.partial.map(|mut p| unsafe { p.as_mut() }) {
            Some(partial) => {
                partial.attach(other);
            }
            None => {
                self.partial = Some(NonNull::from_mut(other.as_head()));
            }
        };
    }

    /// Remove partial slab entry from the cache.
    ///
    /// # Safety
    ///
    /// This function assumes that the entry is part of the partial slab
    /// list.
    pub unsafe fn remove_partial_entry(
        &mut self,
        partial: &SlabDescriptor<T, Partial>,
    ) {
        let target = NonNull::from_ref(partial).addr();
        // SAFETY: the caller guarantees `partial` is in the partial list,
        // so the list has a head.
        let head = unsafe { self.partial.unwrap_unchecked().as_mut() };
        let mut current = head.as_link();

        // SAFETY: every link of the list points at a live node, which
        // nothing else borrows while the cache is locked.
        while let Some(next) = unsafe { current.as_ref() }.next {
            if next.addr() == target {
                // SAFETY: as above.
                unsafe { current.as_mut() }.next = partial.next;
                return;
            }
            current = next;
        }
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
        if let Some(partial) =
            self.partial.map(|mut p| unsafe { p.as_mut() })
        {
            let (allocation, final_state) = partial.alloc();
            match final_state {
                SlabStateKind::Partial => {}
                SlabStateKind::Full => {
                    self.partial = partial.next.map(|next| {
                        // SAFETY: `next` points at a live node of the
                        // partial list, which nothing else borrows while
                        // the cache is locked.
                        NonNull::from_mut(unsafe {
                            next.as_state_mut::<PartialHead>()
                        })
                    });
                    self.attach_or_set_full(
                        partial.morph_mut::<FullDetached>(),
                    );
                }
                // SAFETY: allocating never frees a slab.
                SlabStateKind::Free => unsafe { unreachable_unchecked() },
            }
            return allocation;
        }

        if let Some(free) = self.free.map(|mut p| unsafe { p.as_mut() }) {
            self.free = free.next.map(|next| {
                // SAFETY: `next` points at a live node of the free list,
                // which nothing else borrows while the cache is locked.
                let head = unsafe { next.as_state_mut::<FreeHead>() };
                head.state.set_prev(SlabAddress::default());
                NonNull::from_mut(head)
            });
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

    pub fn dealloc(
        &mut self,
        idx: NonMaxU16,
        slab: &mut SlabDescriptor<T, Used>,
    ) {
        // match slab.is_partial_mut() {
        //     Ok(partial) => {
        //         let state = unsafe { partial.dealloc(idx) };
        //         if let SlabStateKind::Free = state {
        //             unsafe { self.remove_partial_entry(partial) };
        //             match self.free {
        //                 Some(mut free) => unsafe {
        //                     free.as_mut().attach(partial);
        //                 },
        //                 None => {
        //                     self.free = Some(NonNull::from_ref(todo!(
        //                         "create a function partial into free"
        //                     )))
        //                 }
        //             }
        //         }
        //     }
        //     Err(full) => {
        //         full.detach();

        //         let partial: &mut SlabDescriptor<T, Partial> =
        //             unsafe { core::mem::transmute(full) };

        //         partial.state = PartialMeta::new()
        //             .partial(true)
        //             .next_free_idx(u16::MAX)
        //             .total_allocated(T::OBJECT_PER_SLAB as u32);

        //         let state = unsafe { partial.dealloc(idx) };

        //         match state {
        //             SlabStateKind::Free => match self.free {
        //                 Some(mut free) => unsafe {
        //                     free.as_mut().attach(todo!(
        //                         "Create a function partial into free"
        //                     ));
        //                 },
        //                 None => {
        //                     self.free = Some(NonNull::from_ref(todo!(
        //                         "create a function partial into free"
        //                     )))
        //                 }
        //             },
        //             SlabStateKind::Partial => match self.partial {
        //                 Some(mut partial) => unsafe {
        //                     partial.as_mut().attach(todo!(
        //                         "Create a function partial into free"
        //                     ));
        //                 },
        //                 None => {
        //                     self.partial = Some(NonNull::from_ref(todo!(
        //                         "create a function partial into free"
        //                     )))
        //                 }
        //             },
        //             _ => unsafe { unreachable_unchecked() },
        //         }
        //     }
        // };
    }
}
