extern crate alloc;

pub mod free;
pub mod full;
pub mod meta;
pub mod partial;
pub mod state;

pub use state::*;

use crate::{
    slab_address::SlabAddress, slab_objects::SlabObjects, traits::Slab,
};
use core::ptr::NonNull;
use typestate_groups::{Isomorphic, group_impl, group_trait, typestate};

#[typestate(state = S, unsafe_transmute = true)]
pub struct SlabDescriptor<T: Slab, S: SlabState> {
    pub state: S::Meta,
    pub objects: SlabObjects<T>,
    pub next: SlabAddress,
}

pub enum SlabStateKind {
    Partial,
    Free,
    Full,
}

/// The head of a slab list, the first descriptor in it or `None` when the
/// list is empty.
pub(crate) type ListHead<T, H> = Option<NonNull<SlabDescriptor<T, H>>>;

/// A node of a slab list, which links itself into the list at `head` or
/// unlinks itself from it, and returns itself in its new state.
#[group_trait(by = SlabState)]
pub(crate) trait ListNode<T: Slab> {
    /// The state of the list's head.
    type Head: SlabState;
    /// This descriptor in the list.
    type Linked;
    /// This descriptor outside the list.
    type Detached;

    /// Links this descriptor into the list after its head, or as its head
    /// when the list is empty. Either way it comes back as a linked node,
    /// as only the cache's head pointer has the head state.
    fn link(
        &mut self,
        head: &mut ListHead<T, Self::Head>,
    ) -> &mut Self::Linked;

    /// Unlinks this descriptor from the list, moving the head on when this
    /// descriptor is it.
    fn unlink(
        &mut self,
        head: &mut ListHead<T, Self::Head>,
    ) -> &mut Self::Detached;
}

/// The singly-linked partial list. A node doesn't know its previous node,
/// so unlinking one other than the head walks the list from the head.
#[group_impl(PartialStates, state = S)]
impl<T: Slab, S: ListState> ListNode<T> for SlabDescriptor<T, S> {
    type Head = S::Head;
    type Linked = SlabDescriptor<T, S::Linked>;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn link(
        &mut self,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Linked> {
        match head {
            None => {
                *head = Some(NonNull::from_mut(
                    self.cast_state_mut::<S::Head>(),
                ))
            }
            Some(first) => {
                // SAFETY: the head is live and isn't this descriptor,
                // which is outside the list, and nothing else borrows it
                // while the list is locked.
                let first = unsafe { first.as_mut() };
                self.next = first.next;
                first.next =
                    SlabAddress::from_non_null(NonNull::from_mut(self));
            }
        }
        self.cast_state_mut::<S::Linked>()
    }

    fn unlink(
        &mut self,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        let address = SlabAddress::from_non_null(NonNull::from_mut(self));
        let mut first = head.expect("this descriptor is in the list");

        if SlabAddress::from_non_null(first) == address {
            // SAFETY: `next` links a live node of this list.
            *head = unsafe { self.next.as_non_null::<T, S::Head>() };
        } else {
            // SAFETY: the head is live and isn't this descriptor, and
            // nothing else borrows it while the list is locked.
            let mut link = &mut unsafe { first.as_mut() }.next;
            while *link != address {
                // SAFETY: `link` links a live node of this list before
                // this one, which nothing else borrows while the list is
                // locked.
                let mut node = unsafe { link.as_non_null::<T, S>() }
                    .expect("this descriptor is in the list");
                // SAFETY: as above.
                link = &mut unsafe { node.as_mut() }.next;
            }
            *link = self.next;
        }

        self.next = SlabAddress::default();
        self.cast_state_mut::<S::Detached>()
    }
}

/// The doubly-linked free and full lists. Every node of a list shares
/// this descriptor's metadata, so it reads its neighbours in its own
/// state.
#[group_impl(LinkedStates, state = S)]
impl<T: Slab, S: ListState> ListNode<T> for SlabDescriptor<T, S> {
    type Head = S::Head;
    type Linked = SlabDescriptor<T, S::Linked>;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn link(
        &mut self,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Linked> {
        match head {
            None => {
                *head = Some(NonNull::from_mut(
                    self.cast_state_mut::<S::Head>(),
                ))
            }
            Some(first) => {
                // SAFETY: the head is live and isn't this descriptor,
                // which is outside the list, and nothing else borrows it
                // while the list is locked.
                let first = unsafe { first.as_mut() };
                let address =
                    SlabAddress::from_non_null(NonNull::from_mut(self));

                // SAFETY: `next` links a live node of this list, which
                // nothing else borrows while the list is locked.
                if let Some(mut next) =
                    unsafe { first.next.as_non_null::<T, S>() }
                {
                    // SAFETY: as above.
                    unsafe { next.as_mut() }.state.set_prev(address);
                }

                self.next = first.next;
                self.state.set_prev(SlabAddress::from_non_null(
                    NonNull::from_mut(first),
                ));
                first.next = address;
            }
        }
        self.cast_state_mut::<S::Linked>()
    }

    fn unlink(
        &mut self,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        let address = SlabAddress::from_non_null(NonNull::from_mut(self));
        if head.is_some_and(|first| {
            SlabAddress::from_non_null(first) == address
        }) {
            // SAFETY: `next` links a live node of this list.
            *head = unsafe { self.next.as_non_null::<T, S::Head>() };
        }

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
        self.cast_state_mut::<S::Detached>()
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
