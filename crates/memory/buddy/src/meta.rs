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

/// The `prev` of a block in no free list. No link lives at this address,
/// so a real `prev` never equals it.
pub const UNLINKED: NonNull<Next> = NonNull::dangling();

#[bitfields]
#[derive(FromBytes, IntoBytes, Immutable)]
pub struct BuddyFlags {
    #[flag(flag_type = BuddyOrder)]
    pub order: B7,
    pub allocated: B1,
    #[flag(r)]
    pub reserved: B56,
}

/// A block of the arena, in a free list or not.
#[typestate(unsafe_transmute = true)]
#[derive(Clone, Copy)]
pub struct BuddyMeta<S: BuddyState> {
    pub next: Next,
    /// The `next` link that points at this block, or [`UNLINKED`] while
    /// it is in no free list.
    pub prev: NonNull<Next>,
    pub flags: S::Flags,
}

impl BuddyMeta<Regular> {
    /// A block with `flags`, in no free list.
    pub fn new(flags: BuddyFlags) -> BuddyMeta<Regular> {
        BuddyMeta {
            next: None,
            prev: UNLINKED,
            flags,
        }
    }
}

impl<S: BuddyState> BuddyMeta<S> {
    /// Whether this block is in a free list.
    pub fn is_linked(&self) -> bool { self.prev != UNLINKED }
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

/// A free list, as the link to its first block.
///
/// Blocks in the list point back at this link, so a list must not move
/// while it holds blocks.
#[repr(transparent)]
pub struct FreeList(Next);

const _: () = assert!(size_of::<FreeList>() == size_of::<Next>());

impl FreeList {
    /// A list with no blocks.
    pub const fn empty() -> FreeList { FreeList(None) }

    /// The first block of the list.
    pub fn first(&self) -> Next { self.0 }

    /// The link to the first block, which a block links in after.
    pub fn head_mut(&mut self) -> &mut Next { &mut self.0 }
}

impl Debug for FreeList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list_fmt = f.debug_list();

        let mut next = self.0;
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

// SAFETY: a free list links only blocks of its allocator's arena, which
// the allocator's locks guard.
unsafe impl Send for FreeList {}
