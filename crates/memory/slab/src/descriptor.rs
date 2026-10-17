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
use core::{
    fmt::{self, Debug, Formatter},
    ptr::NonNull,
};
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
///
/// Only [`Link`] and [`Unlink`] set it, so a head that isn't `None`
/// points at a live descriptor in the list.
#[repr(transparent)]
pub(crate) struct ListHead<T: Slab, H: SlabState>(
    Option<NonNull<SlabDescriptor<T, H>>>,
);

impl<T: Slab, H: SlabState> ListHead<T, H> {
    /// A list with no descriptors.
    pub(crate) const fn empty() -> Self { Self(None) }

    /// The first descriptor, or `None` when the list is empty.
    ///
    /// # Safety
    ///
    /// The descriptor must not be used after its page is freed, and the
    /// caller must hold no other reference to it while this one is used.
    pub(crate) unsafe fn first<'a>(
        self,
    ) -> Option<&'a mut SlabDescriptor<T, H>> {
        // SAFETY: only `link` and `unlink` set the head, so it points at
        // a live descriptor, and the caller holds no other reference to
        // it.
        self.0.map(|mut first| unsafe { first.as_mut() })
    }
}

impl<T: Slab, H: SlabState> Clone for ListHead<T, H> {
    fn clone(&self) -> Self { *self }
}

impl<T: Slab, H: SlabState> Copy for ListHead<T, H> {}

impl<T: Slab, H: SlabState> Debug for ListHead<T, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

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
        // SAFETY: this descriptor is outside the list, so it isn't the
        // head, and nothing else references the head while the list is
        // locked.
        match unsafe { head.first() } {
            None => {
                self.state.set_head(true);
                head.0 = Some(NonNull::from_mut(
                    self.cast_state_mut::<S::Head>(),
                ));
            }
            Some(first) => {
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
        // SAFETY: this descriptor is outside the list, so it isn't the
        // head, and nothing else references the head while the list is
        // locked.
        match unsafe { head.first() } {
            None => {
                head.0 = Some(NonNull::from_mut(
                    self.cast_state_mut::<S::Head>(),
                ))
            }
            Some(first) => {
                let address =
                    SlabAddress::from_non_null(NonNull::from_mut(self));

                // SAFETY: `next` links the node after the head, in state
                // `S`, so it is neither the head nor this descriptor,
                // and nothing else references it while the list is
                // locked.
                if let Some(next) =
                    unsafe { first.next.node_mut::<T, S>() }
                {
                    next.state.set_prev(address);
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

        head.0 = self.next.as_non_null::<T, S::Head>();
        // SAFETY: the new head is the node after this one, and nothing
        // else references it while the list is locked.
        if let Some(first) = unsafe { head.first() } {
            first.state.set_head(true);
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

        head.0 = self.next.as_non_null::<T, S::Head>();
        // SAFETY: the new head is the node after this one, and nothing
        // else references it while the list is locked.
        if let Some(first) = unsafe { head.first() } {
            first.state.set_prev(SlabAddress::default());
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
        // SAFETY: an attached node isn't the head, and nothing else
        // references the head while the list is locked.
        let first = unsafe { head.first() }
            .expect("an attached node's list has a head");

        let mut link = &mut first.next;
        while *link != address {
            // SAFETY: `link` links a node of this list before this one,
            // in state `S`, so it isn't this descriptor, and nothing else
            // references it while the list is locked.
            let node = unsafe { link.node_mut::<T, S>() }
                .expect("this descriptor is in the list");
            link = &mut node.next;
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

        // SAFETY: `prev` links the node before this one, in state `S`,
        // which nothing else references while the list is locked.
        unsafe { prev.node_mut::<T, S>() }
            .expect("an attached node has a previous node")
            .next = self.next;

        // SAFETY: `next` links the node after this one, in state `S`,
        // which nothing else references while the list is locked.
        if let Some(next) = unsafe { self.next.node_mut::<T, S>() } {
            next.state.set_prev(prev);
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
