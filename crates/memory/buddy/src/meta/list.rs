//! Linking blocks into and out of a free list.

use common::page_index::{PageIndex, Prev};
use typestate_groups::{group_impl, group_trait};

use super::state::*;
use crate::meta::{BuddyArena, BuddyBlock, BuddyLinks, FreeList};

/// A block outside every free list, which links itself into one.
#[group_trait(by = BuddyState)]
pub trait Link {
    /// Links this block, the one at `index` in `arena`, in as the first
    /// block of `list`.
    fn link<Block: BuddyBlock>(
        &mut self,
        index: PageIndex,
        list: &mut FreeList,
        arena: &impl BuddyArena<Block>,
    ) -> &mut BuddyLinks<Attached>;
}

/// A block in a free list, which unlinks itself and returns itself
/// outside the list.
#[group_trait(by = BuddyState)]
pub trait Unlink {
    /// Unlinks this block from `list`, the free list it is in.
    fn unlink<Block: BuddyBlock>(
        &mut self,
        list: &mut FreeList,
        arena: &impl BuddyArena<Block>,
    ) -> &mut BuddyLinks<Detached>;
}

#[group_impl(BuddyState = Nodes, state = Detached)]
impl Link for BuddyLinks<Detached> {
    fn link<Block: BuddyBlock>(
        &mut self,
        index: PageIndex,
        list: &mut FreeList,
        arena: &impl BuddyArena<Block>,
    ) -> &mut BuddyLinks<Attached> {
        debug_assert!(
            self.next.is_none() && !self.is_linked(),
            "a detached block has no neighbours"
        );
        if let Some(first) = list.first() {
            // SAFETY: `first` is in `list`, and it isn't `self`.
            unsafe { arena.links_at(first).as_mut() }.prev =
                Some(Prev::Page(index));
        }
        self.next = list.first();
        self.prev = Some(Prev::First);
        list.set_first(Some(index));

        self.as_state_mut::<Attached>()
    }
}

/// Unlinks an attached block, through the block or list that points at
/// it.
#[group_impl(BuddyState = Nodes, state = Attached)]
impl Unlink for BuddyLinks<Attached> {
    fn unlink<Block: BuddyBlock>(
        &mut self,
        list: &mut FreeList,
        arena: &impl BuddyArena<Block>,
    ) -> &mut BuddyLinks<Detached> {
        match self.prev {
            Some(Prev::First) => list.set_first(self.next),
            Some(Prev::Page(prev)) => {
                // SAFETY: `prev` is the block before `self` in the list.
                unsafe { arena.links_at(prev).as_mut() }.next = self.next
            }
            None => {
                unreachable!("an attached block is in a list")
            }
        }
        if let Some(next) = self.next {
            // SAFETY: `next` is the block after `self` in the list.
            unsafe { arena.links_at(next).as_mut() }.prev = self.prev;
        }

        self.next = None;
        self.prev = None;
        self.as_state_mut::<Detached>()
    }
}
