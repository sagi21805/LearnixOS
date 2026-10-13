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

/// The list a slab descriptor belongs to, and the states it takes as the
/// list's head, as a node after the head, and outside the list.
#[state_types]
pub trait SlabList {
    type Head: SlabState;
    type Attached: SlabState;
    type Detached: SlabState;
}

/// Where a slab descriptor sits relative to its list.
#[state_types]
pub trait SlabRole {}

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

/// A slab reached through a link, whose state the link doesn't record.
#[state]
pub struct Erased;

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
impl SlabState for (Used, Erased) {
    #[size(8)]
    type Meta = RawMeta;
}

#[group(PartialList)]
impl SlabList for (PartialHead, Partial, PartialDetached) {
    type Head = PartialHead;
    type Attached = Partial;
    type Detached = PartialDetached;
}

#[group(FreeList)]
impl SlabList for (FreeHead, Free, FreeDetached) {
    type Head = FreeHead;
    type Attached = Free;
    type Detached = FreeDetached;
}

#[group(FullList)]
impl SlabList for (FullHead, Full, FullDetached) {
    type Head = FullHead;
    type Attached = Full;
    type Detached = FullDetached;
}

#[group(HeadRole)]
impl SlabRole for (PartialHead, FreeHead, FullHead) {}

#[group(AttachedRole)]
impl SlabRole for (Partial, Free, Full) {}

#[group(DetachedRole)]
impl SlabRole for (PartialDetached, FreeDetached, FullDetached) {}
