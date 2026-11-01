//! Linking blocks into and out of a free list.

use core::ptr::NonNull;

use typestate_groups::{group_impl, group_trait};

use super::state::*;
use crate::meta::{BuddyLinks, Next, UNLINKED};

/// A block outside every free list, which links itself into one.
#[group_trait(by = BuddyState)]
pub trait Link {
    /// Links this block in right after `at`, the `next` link of a free
    /// list's head or of a block in it.
    fn link<'a>(
        &'a mut self,
        at: &mut Next,
    ) -> &'a mut BuddyLinks<Attached>;
}

/// A block in a free list, which unlinks itself and returns itself
/// outside the list.
#[group_trait(by = BuddyState)]
pub trait Unlink {
    /// Unlinks this block from its free list.
    fn unlink(&mut self) -> &mut BuddyLinks<Detached>;
}

#[group_impl(BuddyState = Nodes, state = Detached)]
impl Link for BuddyLinks<Detached> {
    fn link<'a>(
        &'a mut self,
        at: &mut Next,
    ) -> &'a mut BuddyLinks<Attached> {
        debug_assert!(
            self.next.is_none() && !self.is_linked(),
            "a detached block has no neighbours"
        );
        let next = *at;
        let link = NonNull::from_mut(at);

        if let Some(mut next) = next {
            // SAFETY: `next` is the block after `at` in its free list,
            // which isn't `self`.
            unsafe { next.as_mut() }.prev =
                NonNull::from_mut(&mut self.next);
        }
        self.next = next;
        self.prev = link;

        let attached = self.as_state_mut::<Attached>();
        // SAFETY: `link` is `at`, which the caller lends to this call.
        unsafe { link.write(Some(NonNull::from_mut(attached))) };
        attached
    }
}

/// Unlinks an attached block, through the link that points at it.
#[group_impl(BuddyState = Nodes, state = Attached)]
impl Unlink for BuddyLinks<Attached> {
    fn unlink(&mut self) -> &mut BuddyLinks<Detached> {
        debug_assert!(
            self.is_linked(),
            "an attached block has a previous link"
        );
        let prev = self.prev;

        // SAFETY: `prev` is the `next` link of the node before `self`.
        unsafe { prev.write(self.next) };
        if let Some(mut next) = self.next {
            // SAFETY: `next` is the block after `self`, which isn't
            // `self`.
            unsafe { next.as_mut() }.prev = prev;
        }

        self.next = None;
        self.prev = UNLINKED;
        self.as_state_mut::<Detached>()
    }
}
