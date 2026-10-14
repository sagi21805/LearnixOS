use crate::{
    descriptor::{
        FreeDetached, FullDetached, PartialDetached, SlabDescriptor,
        SlabState, SlabStateKind, meta::PartialMeta,
    },
    slab_address::SlabAddress,
    traits::Slab,
};
use core::ptr::NonNull;
use nonmax::NonMaxU16;
use typestate_groups::MorphFrom;

/// A free slab allocating its first object becomes partial, starting at
/// index 0 with nothing allocated.
impl<T: Slab> MorphFrom<SlabDescriptor<T, FreeDetached>>
    for SlabDescriptor<T, PartialDetached>
{
    fn morph_from(src: SlabDescriptor<T, FreeDetached>) -> Self {
        SlabDescriptor {
            state: PartialMeta::default(),
            objects: src.objects,
            next: SlabAddress::default(),
        }
    }
}

/// A full slab freeing an object becomes partial, starting with every
/// object allocated and an empty free list.
impl<T: Slab> MorphFrom<SlabDescriptor<T, FullDetached>>
    for SlabDescriptor<T, PartialDetached>
{
    fn morph_from(src: SlabDescriptor<T, FullDetached>) -> Self {
        SlabDescriptor {
            state: PartialMeta::new()
                .next_free_idx(u16::MAX)
                .total_allocated(T::OBJECT_PER_SLAB as u32)
                .partial(true),
            objects: src.objects,
            next: SlabAddress::default(),
        }
    }
}

impl<T: Slab, S: SlabState<Meta = PartialMeta>> SlabDescriptor<T, S> {
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
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of this slab.
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
