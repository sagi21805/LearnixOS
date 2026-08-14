use super::{FullFreeMeta, Partial, SlabDescriptor, SlabStateKind};
use crate::{
    descriptor::{
        FreeDetached, FullDetached, PartialDetached, PartialMeta,
    },
    traits::{Attach, ConvertInplace, SelfAttach, Slab},
};
use core::ptr::NonNull;
use nonmax::NonMaxU16;

impl<T: Slab> SlabDescriptor<T, Partial> {
    /// Allocate an object from this slab returning the allocated object
    /// and state of the slab.
    ///
    /// # Parameters
    ///
    /// * `head` - The head of the full slab descriptor. This slab will be
    ///   attached to it, if the allocation will make this slab full.
    pub fn alloc(&mut self) -> (NonNull<T>, SlabStateKind) {
        debug_assert!(self.state.is_partial(),);

        let idx = self.state.get_next_free_idx() as usize;
        let preallocated =
            unsafe { self.objects.add(idx as usize).as_mut() };

        let mut final_state = SlabStateKind::Partial;

        self.state
            .set_total_allocated(self.state.get_total_allocated() + 1);

        match unsafe { preallocated.next_free_idx } {
            Some(idx) => {
                self.state.set_next_free_idx(idx.get());
            }
            None => {
                final_state = SlabStateKind::Full;
            }
        }

        unsafe {
            (NonNull::from_mut(&mut preallocated.allocated), final_state)
        }
    }

    /// Deallocate an object  from this slab.
    ///
    /// # Parameters
    ///
    /// * `idx` - The index of the object to deallocate.
    pub unsafe fn dealloc(&mut self, idx: NonMaxU16) -> SlabStateKind {
        debug_assert!(self.state.is_partial());

        unsafe {
            self.objects.add(idx.get() as usize).as_mut().next_free_idx =
                NonMaxU16::new(self.state.get_next_free_idx());
        }

        self.state.set_next_free_idx(idx.get());

        self.state
            .set_total_allocated(self.state.get_total_allocated() - 1);

        if self.state.get_total_allocated() == 0 {
            SlabStateKind::Free
        } else {
            SlabStateKind::Partial
        }
    }
}

impl<T: Slab> Attach<T, Partial> for SlabDescriptor<T, Partial> {
    fn attach_free(
        &mut self,
        _other: &mut SlabDescriptor<T, FreeDetached>,
    ) -> &mut SlabDescriptor<T, Partial> {
        unimplemented!()
    }

    fn attach_full(
        &mut self,
        _other: &mut SlabDescriptor<T, FullDetached>,
    ) -> &mut SlabDescriptor<T, Partial> {
        unimplemented!()
    }

    fn attach_partial(
        &mut self,
        other: &mut SlabDescriptor<T, PartialDetached>,
    ) -> &mut SlabDescriptor<T, Partial> {
        other.next = self.next.map(|p| p.cast());

        self.next = Some(NonNull::from_mut(other).cast());

        unsafe { core::mem::transmute(other) }
    }
}

impl<T: Slab> ConvertInplace<T, FreeDetached, PartialDetached>
    for SlabDescriptor<T, PartialDetached>
{
    fn convert_inplace(
        &mut self,
        meta: FullFreeMeta,
    ) -> &mut SlabDescriptor<T, FreeDetached> {
        debug_assert!(!meta.is_partial());
        let free = unsafe {
            core::mem::transmute::<
                &mut SlabDescriptor<T, PartialDetached>,
                &mut SlabDescriptor<T, FreeDetached>,
            >(self)
        };
        free.state = meta;
        free
    }
}

impl<T: Slab> ConvertInplace<T, FullDetached, PartialDetached>
    for SlabDescriptor<T, PartialDetached>
{
    fn convert_inplace(
        &mut self,
        meta: FullFreeMeta,
    ) -> &mut SlabDescriptor<T, FullDetached> {
        debug_assert!(!meta.is_partial());
        let full = unsafe {
            core::mem::transmute::<
                &mut SlabDescriptor<T, PartialDetached>,
                &mut SlabDescriptor<T, FullDetached>,
            >(self)
        };
        full.state = meta;
        full
    }
}

unsafe impl<T: Slab> SelfAttach<T, PartialDetached>
    for SlabDescriptor<T, PartialDetached>
{
    fn attach_self(&mut self) -> &mut SlabDescriptor<T, Partial> {
        debug_assert!(self.next == None);
        debug_assert!(self.state.is_partial());
        debug_assert!(self.state == PartialMeta::default());
        unsafe { core::mem::transmute(self) }
    }
}
