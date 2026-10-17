//! The states of a [`SlabDescriptor`](super::SlabDescriptor) and the
//! groups they form.

use typestate_groups::{group, state, state_types};

use crate::descriptor::meta::{FullFreeMeta, PartialMeta, RawMeta};

/// The state of a slab descriptor, which sets the type of its metadata.
#[state_types]
pub trait SlabState {
    /// The metadata in `SlabDescriptor::state`.
    type Meta;
}

/// The first node of the partial list.
#[state]
pub struct PartialHead;

/// A slab with some allocated objects and some free ones.
#[state]
pub struct Partial;

/// A partial slab outside the partial list.
#[state]
pub struct PartialDetached;

/// The first node of the free list.
#[state]
pub struct FreeHead;

/// A slab that allocates no objects, whose first free index is 0.
#[state]
pub struct Free;

/// A free slab outside the free list.
#[state]
pub struct FreeDetached;

/// The first node of the full list.
#[state]
pub struct FullHead;

/// A slab whose objects are all allocated.
#[state]
pub struct Full;

/// A full slab outside the full list.
#[state]
pub struct FullDetached;

/// A slab that is full or partial, found when freeing one of its objects.
#[state]
pub struct Used;

#[group(PartialStates)]
impl SlabState for (PartialHead, Partial, PartialDetached) {
    #[size(8)]
    type Meta = PartialMeta;
}

#[group(LinkedStates)]
impl SlabState
    for (FreeHead, Free, FreeDetached, FullHead, Full, FullDetached)
{
    #[size(8)]
    type Meta = FullFreeMeta;
}

#[group(RawStates)]
impl SlabState for (Used,) {
    #[size(8)]
    type Meta = RawMeta;
}

/// The states of the list a state belongs to, all in its group.
pub trait ListState: SlabState {
    /// The state of the list's first node.
    type Head: SlabStateGroupMember<Self::Marker>;
    /// The state of a node in the list.
    type Attached: SlabStateGroupMember<Self::Marker>;
    /// The state of a node outside the list.
    type Detached: SlabStateGroupMember<Self::Marker>;
}

/// Implements [`ListState`] for the head, attached and detached states of
/// one list.
macro_rules! list_states {
    ($head:ident, $attached:ident, $detached:ident) => {
        list_states!(@each ($head, $attached, $detached) $head $attached $detached);
    };
    (@each ($head:ident, $attached:ident, $detached:ident) $($state:ident)+) => {
        $(
            impl ListState for $state {
                type Head = $head;
                type Attached = $attached;
                type Detached = $detached;
            }
        )+
    };
}

list_states!(PartialHead, Partial, PartialDetached);
list_states!(FreeHead, Free, FreeDetached);
list_states!(FullHead, Full, FullDetached);
