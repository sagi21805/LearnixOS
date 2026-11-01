//! The states of a [`BuddyLinks`](super::BuddyLinks) and the groups they
//! form.

use typestate_groups::{group, state, state_types};

/// Where a block stands relative to the free lists.
#[state_types]
pub trait BuddyState {}

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
impl BuddyState for (Regular, Attached, Detached) {}
