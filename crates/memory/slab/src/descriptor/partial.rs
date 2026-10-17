use crate::{
    descriptor::{
        FreeDetached, FullDetached, PartialDetached, SlabDescriptor,
        SlabState, meta::PartialMeta,
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
    /// Takes the first free object, and returns it with the slab's state
    /// afterwards.
    pub(crate) fn take_object(&mut self) -> (NonNull<T>, Taken) {
        let slot =
            self.objects.slot(self.state.get_next_free_idx() as usize);
        // SAFETY: the slot heads this slab's free list, so it is free and
        // its live field is `next_free_idx`, which `repr(C)` puts at its
        // start.
        let next = unsafe { slot.cast::<Option<NonMaxU16>>().read() };

        self.state
            .set_total_allocated(self.state.get_total_allocated() + 1);

        let taken = match next {
            Some(idx) => {
                self.state.set_next_free_idx(idx.get());
                Taken::Partial
            }
            None => Taken::Full,
        };

        (slot.cast::<T>(), taken)
    }

    /// Puts object `idx` back on the free list, and returns the slab's
    /// state afterwards.
    ///
    /// # Safety
    ///
    /// `idx` must be an allocated object of this slab.
    pub(crate) unsafe fn return_object(
        &mut self,
        idx: NonMaxU16,
    ) -> Returned {
        let slot = self.objects.slot(idx.get() as usize);
        let next = NonMaxU16::new(self.state.get_next_free_idx());
        // SAFETY: the caller guarantees `idx` is allocated, and its owner
        // is done with it, so nothing else references its slot. `repr(C)`
        // puts `next_free_idx` at the slot's start.
        unsafe { slot.cast::<Option<NonMaxU16>>().write(next) };

        self.state.set_next_free_idx(idx.get());

        self.state
            .set_total_allocated(self.state.get_total_allocated() - 1);

        if self.state.get_total_allocated() == 0 {
            Returned::Free
        } else {
            Returned::Partial
        }
    }
}

/// What a slab became after giving out an object.
pub(crate) enum Taken {
    Partial,
    Full,
}

/// What a slab became after taking an object back.
pub(crate) enum Returned {
    Partial,
    Free,
}
