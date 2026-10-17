extern crate alloc;

pub mod free;
pub mod full;
pub mod meta;
pub mod partial;
pub mod state;

pub use state::*;

use crate::{
    Generic64, slab_address::SlabAddress, slab_objects::SlabObjects,
    traits::Slab,
};
use core::ptr::NonNull;
use typestate_groups::{Isomorphic, group_impl, group_trait, typestate};

#[typestate(state = S, unsafe_transmute = true)]
pub struct SlabDescriptor<T: Slab, S: SlabState> {
    pub state: S::Meta,
    pub objects: SlabObjects<T>,
    pub next: SlabAddress,
}

// A descriptor is its metadata, its objects pointer and its `next` link,
// in every state.
const _: () = {
    assert!(size_of::<SlabDescriptor<Generic64, Partial>>() == 24);
    assert!(size_of::<SlabDescriptor<Generic64, Free>>() == 24);
};

/// The head of a slab list, the first descriptor in it or `None` when the
/// list is empty.
pub(crate) type ListHead<T, H> = Option<NonNull<SlabDescriptor<T, H>>>;

/// A slab outside its list, which links itself into the list at `head`.
#[group_trait(by = (SlabState, ListRole))]
pub(crate) trait Link<T: Slab> {
    /// The state of the list's head.
    type Head: SlabState;

    /// Links this descriptor into the list after its head, or as its head
    /// when the list is empty.
    fn link(&mut self, head: &mut ListHead<T, Self::Head>);
}

/// A slab in its list, which unlinks itself from the list at `head` and
/// returns itself outside the list.
#[group_trait(by = (SlabState, ListRole))]
pub(crate) trait Unlink<T: Slab> {
    /// The state of the list's head.
    type Head: SlabState;
    /// This descriptor outside the list.
    type Detached;

    /// Unlinks this descriptor from the list.
    fn unlink(
        &mut self,
        head: &mut ListHead<T, Self::Head>,
    ) -> &mut Self::Detached;
}

/// Links a slab into the singly-linked partial list, marking it as the
/// head when the list is empty.
#[group_impl(SlabState = SinglyLinked, ListRole = Detached, state = S)]
impl<T: Slab, S: ListState> Link<T> for SlabDescriptor<T, S> {
    type Head = S::Head;

    fn link(&mut self, head: &mut ListHead<T, S::Head>) {
        match head {
            None => {
                self.state.set_head(true);
                *head = Some(NonNull::from_mut(
                    self.cast_state_mut::<S::Head>(),
                ));
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
    }
}

/// Links a slab into a doubly-linked free or full list. Every node of a
/// list shares this descriptor's metadata, so it reads its neighbours in
/// its own state.
#[group_impl(SlabState = DoublyLinked, ListRole = Detached, state = S)]
impl<T: Slab, S: ListState> Link<T> for SlabDescriptor<T, S> {
    type Head = S::Head;

    fn link(&mut self, head: &mut ListHead<T, S::Head>) {
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

                if let Some(mut next) = first.next.as_non_null::<T, S>() {
                    // SAFETY: `next` links a live node of this list, in
                    // state `S`, which nothing else borrows while the
                    // list is locked.
                    unsafe { next.as_mut() }.state.set_prev(address);
                }

                self.next = first.next;
                self.state.set_prev(SlabAddress::from_non_null(
                    NonNull::from_mut(first),
                ));
                first.next = address;
            }
        }
    }
}

/// Unlinks the head of the singly-linked partial list, and marks the
/// next node as the head.
#[group_impl(SlabState = SinglyLinked, ListRole = Heads, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S> {
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        debug_assert!(
            self.state.is_head(),
            "the head has its head bit set"
        );

        *head = self.next.as_non_null::<T, S::Head>();
        if let Some(mut first) = *head {
            // SAFETY: `next` linked a live node of this list, which is
            // now its head and which nothing else borrows while the list
            // is locked.
            unsafe { first.as_mut() }.state.set_head(true);
        }

        self.next = SlabAddress::default();
        self.state.set_head(false);
        self.cast_state_mut::<S::Detached>()
    }
}

/// Unlinks the head of a doubly-linked free or full list, and leaves the
/// next node without a previous one.
#[group_impl(SlabState = DoublyLinked, ListRole = Heads, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S> {
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        debug_assert!(
            self.state.get_prev() == SlabAddress::default(),
            "the head has no previous node"
        );

        *head = self.next.as_non_null::<T, S::Head>();
        if let Some(mut first) = *head {
            // SAFETY: `next` linked a live node of this list, which is
            // now its head and which nothing else borrows while the list
            // is locked.
            unsafe { first.as_mut() }
                .state
                .set_prev(SlabAddress::default());
        }

        self.next = SlabAddress::default();
        self.cast_state_mut::<S::Detached>()
    }
}

/// Unlinks a node of the singly-linked partial list other than its head.
/// A node doesn't know its previous node, so it walks the list from the
/// head.
#[group_impl(SlabState = SinglyLinked, ListRole = Attached, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S> {
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        debug_assert!(
            !self.state.is_head(),
            "an attached node isn't the head"
        );
        let address = SlabAddress::from_non_null(NonNull::from_mut(self));
        let mut first = head.expect("an attached node's list has a head");

        // SAFETY: the head is live and isn't this descriptor, and nothing
        // else borrows it while the list is locked.
        let mut link = &mut unsafe { first.as_mut() }.next;
        while *link != address {
            let mut node = link
                .as_non_null::<T, S>()
                .expect("this descriptor is in the list");
            // SAFETY: `link` links a live node of this list before this
            // one, in state `S`, which nothing else borrows while the
            // list is locked.
            link = &mut unsafe { node.as_mut() }.next;
        }
        *link = self.next;

        self.next = SlabAddress::default();
        self.cast_state_mut::<S::Detached>()
    }
}

/// Unlinks a node of a doubly-linked free or full list other than its
/// head, so it always has a previous node and leaves the head alone.
#[group_impl(SlabState = DoublyLinked, ListRole = Attached, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S> {
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        _head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        let prev = self.state.get_prev();

        let mut prev_node = prev
            .as_non_null::<T, S>()
            .expect("an attached node has a previous node");
        // SAFETY: `prev` links a live node of this list, in state `S`,
        // which nothing else borrows while the list is locked.
        unsafe { prev_node.as_mut() }.next = self.next;

        if let Some(mut next) = self.next.as_non_null::<T, S>() {
            // SAFETY: `next` links a live node of this list, in state
            // `S`, which nothing else borrows while the list is locked.
            unsafe { next.as_mut() }.state.set_prev(prev);
        }

        self.next = SlabAddress::default();
        self.state.set_prev(SlabAddress::default());
        self.cast_state_mut::<S::Detached>()
    }
}

/// A used slab in the state of its list and its place in it.
///
/// The head and attached states hold only while the cache stays locked,
/// because popping the head makes the next node the head.
pub enum UsedSlab<'a, T: Slab> {
    PartialHead(&'a mut SlabDescriptor<T, PartialHead>),
    Partial(&'a mut SlabDescriptor<T, Partial>),
    FullHead(&'a mut SlabDescriptor<T, FullHead>),
    Full(&'a mut SlabDescriptor<T, Full>),
}

impl<T: Slab> SlabDescriptor<T, Used> {
    /// Reads this slab's list from its partial bit, and whether it heads
    /// that list from its head bit or, for a full slab, from it having no
    /// previous node.
    pub fn classify_mut(&mut self) -> UsedSlab<'_, T> {
        if self.state.is_partial() {
            let partial = self.cast_state_mut::<Partial>();
            if partial.state.is_head() {
                UsedSlab::PartialHead(
                    partial.cast_state_mut::<PartialHead>(),
                )
            } else {
                UsedSlab::Partial(partial)
            }
        } else {
            let full = self.cast_state_mut::<Full>();
            if full.state.get_prev() == SlabAddress::default() {
                UsedSlab::FullHead(full.cast_state_mut::<FullHead>())
            } else {
                UsedSlab::Full(full)
            }
        }
    }
}
