//! Linking blocks into and out of a free list.

use core::ptr::NonNull;

use typestate_groups::{
    Isomorphic, MorphFrom, Morphic, group_impl, group_trait,
};

use super::state::*;
use crate::meta::{BuddyBlock, BuddyMeta};

/// A node of a free list, which links a block in right after itself.
#[group_trait(by = ListRole)]
pub trait Link {
    /// Links `block` into this node's free list, right after this node.
    fn link<'a>(
        &mut self,
        block: &'a mut BuddyMeta<Detached>,
    ) -> &'a mut BuddyMeta<Attached>;
}

/// A block in a free list, which unlinks itself and returns itself
/// outside the list.
#[group_trait(by = (BuddyState, ListRole))]
pub trait Unlink {
    /// Unlinks this block from its free list.
    fn unlink(&mut self) -> &mut BuddyMeta<Detached>;
}

/// Links a block after a head or a block. Both keep the link to the next
/// block in `next`, so one impl covers them.
#[group_impl(Linked, state = S)]
impl<S: BuddyState + ListRole> Link for BuddyMeta<S> {
    fn link<'a>(
        &mut self,
        block: &'a mut BuddyMeta<Detached>,
    ) -> &'a mut BuddyMeta<Attached> {
        debug_assert!(
            block.next.is_none() && block.prev.is_none(),
            "a detached block has no neighbours"
        );
        let next = self.next;
        let link = NonNull::from_mut(&mut self.next);

        if let Some(mut next) = next {
            // SAFETY: `next` is the block after `self` in its free list,
            // which isn't `block`.
            unsafe { next.as_mut() }.prev =
                Some(NonNull::from_mut(&mut block.next));
        }
        block.next = next;
        block.prev = Some(link);

        let attached = block.cast_state_mut::<Attached>();
        // SAFETY: `link` is `self.next`, which `self` lends to this call.
        unsafe { link.write(Some(NonNull::from_mut(attached))) };
        attached
    }
}

/// Unlinks an attached block, through the link that points at it.
#[group_impl(BuddyState = Nodes, ListRole = Linked, state = S)]
impl<S: BuddyState + ListRole> Unlink for BuddyMeta<S>
where
    BuddyMeta<Detached>: MorphFrom<BuddyMeta<S>>,
{
    fn unlink(&mut self) -> &mut BuddyMeta<Detached> {
        debug_assert!(
            self.prev.is_some(),
            "an attached block has a previous link"
        );
        // SAFETY: an attached block always has a previous link.
        let prev = unsafe { self.prev.unwrap_unchecked() };

        // SAFETY: `prev` is the `next` link of the node before `self`.
        unsafe { prev.write(self.next) };
        if let Some(mut next) = self.next {
            // SAFETY: `next` is the block after `self`, which isn't
            // `self`.
            unsafe { next.as_mut() }.prev = Some(prev);
        }

        self.morph_mut::<Detached>()
    }
}

/// A block leaving its free list keeps its flags, without its links.
impl MorphFrom<BuddyMeta<Attached>> for BuddyMeta<Detached> {
    fn morph_from(src: BuddyMeta<Attached>) -> Self {
        BuddyMeta {
            next: None,
            prev: None,
            flags: src.flags,
        }
    }
}

// TODO: remove once lib.rs links typed states.
impl BuddyMeta<Head> {
    /// Links `block` into this free list, right after the head.
    #[inline]
    pub fn attach_block<Block: const BuddyBlock>(
        &mut self,
        mut block: NonNull<Block>,
    ) {
        // SAFETY: the allocator only attaches live blocks of its arena
        // that are in no free list.
        let meta = unsafe { block.as_mut() }.meta_mut();
        self.link(meta.cast_state_mut::<Detached>());
    }
}

// TODO: remove once lib.rs links typed states.
impl BuddyMeta<Regular> {
    /// Links `block` into this block's free list, right after this block.
    #[inline]
    pub fn attach_block<Block: const BuddyBlock>(
        &mut self,
        mut block: NonNull<Block>,
    ) {
        // SAFETY: the allocator only attaches live blocks of its arena
        // that are in no free list.
        let meta = unsafe { block.as_mut() }.meta_mut();
        self.cast_state_mut::<Attached>()
            .link(meta.cast_state_mut::<Detached>());
    }

    /// Unlinks this block from its free list, and does nothing when it is
    /// in none, because the allocator detaches some blocks twice.
    pub fn detach(&mut self) -> NonNull<BuddyMeta<Regular>> {
        if self.prev.is_some() {
            self.cast_state_mut::<Attached>().unlink();
        }
        NonNull::from_mut(self)
    }
}
