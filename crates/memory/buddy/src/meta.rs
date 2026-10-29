mod arena;
mod list;
mod state;

pub use arena::*;
pub use list::*;
pub use state::*;

use core::{fmt::Debug, ptr::NonNull};

use bitfields::bitfields;
use common::enums::BuddyOrder;
use typestate_groups::typestate;
use zerocopy::{FromBytes, Immutable, IntoBytes};

/// The link from a node to the first free block after it.
pub type Next = Option<NonNull<BuddyMeta<Attached>>>;

#[bitfields]
#[derive(FromBytes, IntoBytes, Immutable)]
pub struct BuddyFlags {
    #[flag(flag_type = BuddyOrder)]
    pub order: B7,
    pub allocated: B1,
    #[flag(r)]
    pub reserved: B56,
}

/// A node of a free list, the list's head or a block of the arena.
#[typestate(unsafe_transmute = true)]
#[derive(Clone, Copy)]
pub struct BuddyMeta<S: BuddyState> {
    pub next: Next,
    /// The `next` link that points at this block, while it is in a free
    /// list.
    pub prev: Option<NonNull<Next>>,
    pub flags: S::Flags,
}

impl Default for BuddyMeta<Head> {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
            flags: (),
        }
    }
}

impl BuddyMeta<Regular> {
    /// A block with `flags`, in no free list.
    pub fn new(flags: BuddyFlags) -> BuddyMeta<Regular> {
        BuddyMeta {
            next: None,
            prev: None,
            flags,
        }
    }
}

impl<S: BuddyStateGroupMember<Nodes>> Debug for BuddyMeta<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("BuddyMeta");

        ds.field("next", &self.next)
            .field("prev", &self.prev)
            .field("flags", &self.flags);

        #[cfg(debug_assertions)]
        ds.field("self_ptr", &NonNull::from_ref(self));

        ds.finish()
    }
}

impl Debug for BuddyMeta<Head> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list_fmt = f.debug_list();

        let mut next = self.next;
        let mut i = 0;
        while let Some(n) = next {
            list_fmt.entry(unsafe { n.as_ref() });
            next = unsafe { n.as_ref().next };
            if i == 32 {
                return list_fmt.finish();
            }
            i += 1;
        }

        list_fmt.finish()
    }
}

// SAFETY: a node's links point only into its allocator's arena and free
// lists, which the allocator's locks guard.
unsafe impl<S: BuddyState> Send for BuddyMeta<S> {}
