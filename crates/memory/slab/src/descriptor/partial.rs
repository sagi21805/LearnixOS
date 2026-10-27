use crate::{
    descriptor::{
        FreeDetached, FullDetached, ListState, PartialDetached,
        SlabDescriptor, SlabState, meta::PartialMeta,
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

/// A slab leaving the partial list keeps its slots, without its head
/// bit or next node.
impl<T: Slab, S> MorphFrom<SlabDescriptor<T, S>>
    for SlabDescriptor<T, PartialDetached>
where
    S: ListState<Meta = PartialMeta, Detached = PartialDetached>,
{
    fn morph_from(src: SlabDescriptor<T, S>) -> Self {
        SlabDescriptor {
            state: src.state.head(false),
            objects: src.objects,
            next: SlabAddress::default(),
        }
    }
}

impl<T: Slab, S: SlabState<Meta = PartialMeta>> SlabDescriptor<T, S> {
    /// Allocates the first free slot, and returns its object with the
    /// slab's state afterwards.
    pub(crate) fn allocate_slot(&mut self) -> (NonNull<T>, Allocated) {
        let slot = unsafe {
            self.objects.slot(self.state.get_next_free_idx() as usize)
        };
        // SAFETY: the free-list head is free, and `repr(C)` puts
        // `next_free_idx` first.
        let next = unsafe { slot.cast::<Option<NonMaxU16>>().read() };

        self.state
            .set_total_allocated(self.state.get_total_allocated() + 1);

        let taken = match next {
            Some(idx) => {
                self.state.set_next_free_idx(idx.get());
                Allocated::Partial
            }
            None => Allocated::Full,
        };

        (slot.cast::<T>(), taken)
    }

    /// Puts object `idx` back on the free list, and returns the slab's
    /// state afterwards.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of this slab.
    pub(crate) unsafe fn deallocate_slot(
        &mut self,
        idx: NonMaxU16,
    ) -> Deallocated {
        let slot = unsafe { self.objects.slot(idx.get() as usize) };
        let next = NonMaxU16::new(self.state.get_next_free_idx());
        // SAFETY: the owner is done with slot `idx`, and `repr(C)` puts
        // `next_free_idx` first.
        unsafe { slot.cast::<Option<NonMaxU16>>().write(next) };

        self.state.set_next_free_idx(idx.get());

        self.state
            .set_total_allocated(self.state.get_total_allocated() - 1);

        if self.state.get_total_allocated() == 0 {
            Deallocated::Free
        } else {
            Deallocated::Partial
        }
    }
}

/// What a slab became after allocating an object.
pub(crate) enum Allocated {
    Partial,
    Full,
}

/// What a slab became after dellocating an object.
pub(crate) enum Deallocated {
    Partial,
    Free,
}
