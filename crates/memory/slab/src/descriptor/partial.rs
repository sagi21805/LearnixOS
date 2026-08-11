use super::{Free, FullFreeMeta, Partial, SlabDescriptor, SlabStateKind};
use crate::traits::Slab;
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
        debug_assert!(
            self.state.is_partial(),
            "Compiletime state does not match runtime state"
        );

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
    fn attach(&mut self, other: &mut SlabDescriptor<T, Partial>) {
        other.next = self.next;

        self.next = Some(NonNull::from_mut(other));
    }
}

impl<T: Slab> ConvertInplace<T, Free> for SlabDescriptor<T, Partial> {
    fn convert_inplace(
        &mut self,
        meta: FullFreeMeta,
    ) -> &mut SlabDescriptor<T, Free> {
        let free: &mut SlabDescriptor<T, Free> =
            unsafe { core::mem::transmute(self) };

        assert!(!meta.is_partial());

        free.state = meta;

        free
    }
}
