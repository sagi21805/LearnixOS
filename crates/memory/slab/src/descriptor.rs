extern crate alloc;

pub mod free;
pub mod full;
pub mod meta;
pub mod partial;
pub mod state;

pub use state::*;

use crate::{
    descriptor::meta::FullFreeMeta, slab_address::SlabAddress,
    slab_objects::SlabObjects, traits::Slab,
};
use core::ptr::NonNull;
use typestate_groups::{Isomorphic, typestate};

#[typestate(state = S, unsafe_transmute = true)]
pub struct SlabDescriptor<T: Slab, S: SlabState> {
    pub state: S::Meta,

    // TODO: Check the possibility to not save the length here because it
    // is already managed by a freelist so the len may not be needed.
    //
    // Moreover the local T holds the const order in which we allocate for
    // the array so the size can always be calculated.
    pub objects: SlabObjects<T>,
    pub next: SlabAddress,
}

pub enum SlabStateKind {
    Partial,
    Free,
    Full,
}

/// The doubly-linked free and full lists. Every node of a list shares
/// this descriptor's metadata, so it reads its neighbours in its own
/// state.
impl<T: Slab, S: SlabState<Meta = FullFreeMeta>> SlabDescriptor<T, S> {
    /// Links `other` after this descriptor.
    pub(crate) fn link<D: SlabState<Meta = FullFreeMeta>>(
        &mut self,
        other: &mut SlabDescriptor<T, D>,
    ) {
        let other_address =
            SlabAddress::from_non_null(NonNull::from_mut(other));

        // SAFETY: `next` links a live node of this list, which nothing
        // else borrows while the list is locked.
        if let Some(mut next) = unsafe { self.next.as_non_null::<T, S>() }
        {
            // SAFETY: as above.
            unsafe { next.as_mut() }.state.set_prev(other_address);
        }

        other.next = self.next;
        other
            .state
            .set_prev(SlabAddress::from_non_null(NonNull::from_mut(self)));
        self.next = other_address;
    }

    /// Unlinks this descriptor from its list.
    pub(crate) fn unlink(&mut self) {
        let prev = self.state.get_prev();

        // SAFETY: `next` links a live node of this list, which nothing
        // else borrows while the list is locked.
        if let Some(mut next) = unsafe { self.next.as_non_null::<T, S>() }
        {
            // SAFETY: as above.
            unsafe { next.as_mut() }.state.set_prev(prev);
        }

        // SAFETY: `prev` links a live node of this list, which nothing
        // else borrows while the list is locked.
        if let Some(mut prev) = unsafe { prev.as_non_null::<T, S>() } {
            // SAFETY: as above.
            unsafe { prev.as_mut() }.next = self.next;
        }

        self.next = SlabAddress::default();
        self.state.set_prev(SlabAddress::default());
    }
}

impl<T: Slab> SlabDescriptor<T, Used> {
    pub fn is_partial(
        &self,
    ) -> Result<&SlabDescriptor<T, Partial>, &SlabDescriptor<T, Full>>
    {
        if self.state.is_partial() {
            Ok(self.cast_state_ref::<Partial>())
        } else {
            Err(self.cast_state_ref::<Full>())
        }
    }

    pub fn is_partial_mut(
        &mut self,
    ) -> Result<
        &mut SlabDescriptor<T, Partial>,
        &mut SlabDescriptor<T, Full>,
    > {
        if self.state.is_partial() {
            Ok(self.cast_state_mut::<Partial>())
        } else {
            Err(self.cast_state_mut::<Full>())
        }
    }
}
