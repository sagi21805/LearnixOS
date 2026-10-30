//! The states of a [`BuddyMeta`](super::BuddyMeta) and the groups they
//! form.

use typestate_groups::{group, state, state_types};

use crate::meta::BuddyFlags;

/// The state of a block, which sets the type of its flags.
#[state_types]
pub trait BuddyState {
    /// The block's order and allocation bit.
    type Flags;
}

/// A block in a free list.
#[state]
#[derive(Clone, Copy)]
pub struct Attached;

/// A block outside every free list.
#[state]
#[derive(Clone, Copy)]
pub struct Detached;

/// A block the arena handed out, whether in a free list or not.
#[state]
#[derive(Clone, Copy)]
pub struct Regular;

#[group(Nodes)]
impl BuddyState for (Regular, Attached, Detached) {
    #[size(8)]
    type Flags = BuddyFlags;
}
