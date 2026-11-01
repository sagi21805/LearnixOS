extern crate alloc;

pub mod free;
pub mod full;
pub mod meta;
pub mod partial;
pub mod state;

pub use state::*;

use crate::{
    Generic64, SlabArena, SlabFlags,
    traits::{Slab, Unknown},
};
use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ptr::NonNull,
};

use common::page_index::PageIndex;
use typestate_groups::{
    Isomorphic, MorphFrom, Morphic, group_impl, group_trait, typestate,
};

#[typestate(state = S, unsafe_transmute = true)]
pub struct SlabDescriptor<T: Slab, S: SlabState> {
    /// The next node of the slab's list.
    pub next: Option<PageIndex>,
    pub state: S::Meta,
    _object: PhantomData<fn() -> T>,
}

// A descriptor is its `next` link and its metadata, in every state.
const _: () = {
    assert!(size_of::<SlabDescriptor<Generic64, Partial>>() == 8);
    assert!(size_of::<SlabDescriptor<Generic64, Free>>() == 8);
    assert!(size_of::<SlabDescriptor<Unknown, Used>>() == 8);
};

impl SlabDescriptor<Unknown, Unused> {
    /// Writes `descriptor` into this storage, without reading or dropping
    /// what it held, and returns it as its own type.
    ///
    /// # Safety
    ///
    /// The storage must not hold a descriptor that is still in use, so
    /// nothing else reaches it while the returned reference lives.
    pub unsafe fn assign<T: Slab, S: SlabState>(
        &mut self,
        descriptor: SlabDescriptor<T, S>,
    ) -> &mut SlabDescriptor<T, S> {
        const {
            assert!(
                size_of::<SlabDescriptor<T, S>>() == size_of::<Self>()
            );
            assert!(
                align_of::<SlabDescriptor<T, S>>() == align_of::<Self>()
            );
        };
        let mut storage =
            NonNull::from_mut(self).cast::<SlabDescriptor<T, S>>();
        // SAFETY: the storage fits a descriptor of any type.
        unsafe { storage.write(descriptor) };
        // SAFETY: the caller guarantees nothing else uses the storage.
        unsafe { storage.as_mut() }
    }
}

/// The head of a slab list, the index of the first descriptor in it or
/// `None` when the list is empty.
///
/// Only [`Link`] and [`Unlink`] set it, so a head that isn't `None` is
/// the index of a live descriptor in the list.
#[repr(transparent)]
pub(crate) struct ListHead<T: Slab, H: SlabState>(
    Option<PageIndex>,
    PhantomData<*mut SlabDescriptor<T, H>>,
);

impl<T: Slab, H: SlabState> ListHead<T, H> {
    /// A list with no descriptors.
    pub(crate) const fn empty() -> Self { Self(None, PhantomData) }

    /// The first descriptor, or `None` when the list is empty.
    ///
    /// # Safety
    ///
    /// The descriptor must not be used after its page is freed, and the
    /// caller must hold no other reference to it while this one is used.
    pub(crate) unsafe fn first<'a>(
        self,
        arena: &impl SlabArena,
    ) -> Option<&'a mut SlabDescriptor<T, H>> {
        // SAFETY: only `link` and `unlink` set the head, so it is live.
        unsafe { arena.node_mut(self.0) }
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
    /// when the list is empty, and sets the slab's list bits.
    fn link(
        &mut self,
        arena: &impl SlabArena,
        head: &mut ListHead<T, Self::Head>,
    );
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
        arena: &impl SlabArena,
        head: &mut ListHead<T, Self::Head>,
    ) -> &mut Self::Detached;
}

/// A slab in the list headed by `H`, which unlinks itself from it.
pub(crate) trait InList<T: Slab, H: ListState>:
    Unlink<T, Head = H, Detached = SlabDescriptor<T, H::Detached>>
{
}

impl<T: Slab, H: ListState, D> InList<T, H> for D where
    D: Unlink<T, Head = H, Detached = SlabDescriptor<T, H::Detached>>
{
}

/// Links a slab into the singly-linked partial list, marking it as the
/// head when the list is empty.
#[group_impl(SlabState = SinglyLinked, ListRole = Detached, state = S)]
impl<T: Slab, S: ListState> Link<T> for SlabDescriptor<T, S> {
    type Head = S::Head;

    fn link(
        &mut self,
        arena: &impl SlabArena,
        head: &mut ListHead<T, S::Head>,
    ) {
        debug_assert!(
            self.next.is_none(),
            "a detached node has no next node"
        );
        let index = arena.index_of(self);
        // SAFETY: a detached node isn't the head.
        let first = unsafe { head.first(arena) };
        arena.set_slab_flags_of(
            self,
            SlabFlags::new().partial(true).head(first.is_none()),
        );
        match first {
            None => head.0 = Some(index),
            Some(first) => {
                self.next = first.next;
                first.next = Some(index);
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

    fn link(
        &mut self,
        arena: &impl SlabArena,
        head: &mut ListHead<T, S::Head>,
    ) {
        debug_assert!(
            self.next.is_none() && self.state.prev().is_none(),
            "a detached node has no neighbours"
        );
        let index = arena.index_of(self);
        arena.set_slab_flags_of(self, SlabFlags::new());
        // SAFETY: a detached node isn't the head.
        match unsafe { head.first(arena) } {
            None => head.0 = Some(index),
            Some(first) => {
                // SAFETY: `next` is the node after the head, not `self`.
                let next = unsafe { arena.node_mut::<T, S>(first.next) };
                if let Some(next) = next {
                    next.state.set_prev(Some(index));
                }

                self.next = first.next;
                self.state.set_prev(Some(arena.index_of(first)));
                first.next = Some(index);
            }
        }
    }
}

/// Unlinks the head of the singly-linked partial list, and marks the
/// next node as the head.
#[group_impl(SlabState = SinglyLinked, ListRole = Heads, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S>
where
    SlabDescriptor<T, S::Detached>: MorphFrom<SlabDescriptor<T, S>>,
{
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        arena: &impl SlabArena,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        debug_assert!(
            arena.slab_flags_of(self).is_head()
                && head.0 == Some(arena.index_of(self)),
            "the list starts at its head, whose head bit is set"
        );

        head.0 = self.next;
        // SAFETY: the new head is the node after `self`.
        if let Some(first) = unsafe { head.first(arena) } {
            arena.set_slab_flags_of(
                first,
                SlabFlags::new().partial(true).head(true),
            );
        }

        self.morph_mut::<S::Detached>()
    }
}

/// Unlinks the head of a doubly-linked free or full list, and leaves the
/// next node without a previous one.
#[group_impl(SlabState = DoublyLinked, ListRole = Heads, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S>
where
    SlabDescriptor<T, S::Detached>: MorphFrom<SlabDescriptor<T, S>>,
{
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        arena: &impl SlabArena,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        debug_assert!(
            self.state.prev().is_none()
                && head.0 == Some(arena.index_of(self)),
            "the list starts at its head, which has no previous node"
        );

        head.0 = self.next;
        // SAFETY: the new head is the node after `self`.
        if let Some(first) = unsafe { head.first(arena) } {
            first.state.set_prev(None);
        }

        self.morph_mut::<S::Detached>()
    }
}

/// Unlinks a node of the singly-linked partial list other than its head.
/// A node doesn't know its previous node, so it walks the list from the
/// head.
#[group_impl(SlabState = SinglyLinked, ListRole = Attached, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S>
where
    SlabDescriptor<T, S::Detached>: MorphFrom<SlabDescriptor<T, S>>,
{
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        arena: &impl SlabArena,
        head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        debug_assert!(
            !arena.slab_flags_of(self).is_head(),
            "an attached node isn't the head"
        );
        let index = Some(arena.index_of(self));
        // SAFETY: an attached node isn't the head, and its list has one.
        let first = unsafe { head.first(arena).unwrap_unchecked() };

        let mut link = &mut first.next;
        while *link != index {
            // SAFETY: `link` points at a node before `self`, which is in
            // the list.
            let node = unsafe {
                arena.node_mut::<T, S>(*link).unwrap_unchecked()
            };
            link = &mut node.next;
        }
        *link = self.next;

        self.morph_mut::<S::Detached>()
    }
}

/// Unlinks a node of a doubly-linked free or full list other than its
/// head, so it always has a previous node and leaves the head alone.
#[group_impl(SlabState = DoublyLinked, ListRole = Attached, state = S)]
impl<T: Slab, S: ListState> Unlink<T> for SlabDescriptor<T, S>
where
    SlabDescriptor<T, S::Detached>: MorphFrom<SlabDescriptor<T, S>>,
{
    type Head = S::Head;
    type Detached = SlabDescriptor<T, S::Detached>;

    fn unlink(
        &mut self,
        arena: &impl SlabArena,
        _head: &mut ListHead<T, S::Head>,
    ) -> &mut SlabDescriptor<T, S::Detached> {
        let prev = self.state.prev();

        // SAFETY: `prev` is the node before `self`, which an attached node
        // always has.
        unsafe { arena.node_mut::<T, S>(prev).unwrap_unchecked() }.next =
            self.next;

        // SAFETY: `next` is the node after `self`.
        if let Some(next) = unsafe { arena.node_mut::<T, S>(self.next) } {
            next.state.set_prev(prev);
        }

        self.morph_mut::<S::Detached>()
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
    pub fn classify_mut(
        &mut self,
        arena: &impl SlabArena,
    ) -> UsedSlab<'_, T> {
        let flags = arena.slab_flags_of(self);
        if flags.is_partial() {
            let partial = self.cast_state_mut::<Partial>();
            if flags.is_head() {
                UsedSlab::PartialHead(
                    partial.cast_state_mut::<PartialHead>(),
                )
            } else {
                UsedSlab::Partial(partial)
            }
        } else {
            let full = self.cast_state_mut::<Full>();
            if full.state.prev().is_none() {
                UsedSlab::FullHead(full.cast_state_mut::<FullHead>())
            } else {
                UsedSlab::Full(full)
            }
        }
    }
}
