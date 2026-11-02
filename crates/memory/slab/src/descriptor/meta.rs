#![allow(
    non_snake_case,
    reason = "_next_free_idx inside PartialMeta generates funtions with \
              names that contain '__' which is not snake_case"
)]

use bitfields::bitfields;
use common::page_index::Prev;
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::object_index::{ObjectIndexBits, ObjectIndex};

/// The metadata of a partial slab, its free list and how many objects it
/// allocates.
///
/// It stores the bits of the first free slot, an `Option<ObjectIndex>`,
/// because a descriptor's metadata must take any bit pattern for the
/// casts between its states.
#[bitfields]
#[derive(PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
pub struct PartialMeta {
    _next_free_idx: B16,
    pub total_allocated: B16,
}

impl PartialMeta {
    pub fn next_free_idx(self, v: Option<ObjectIndex>) -> Self {
        self._next_free_idx(v.to_bits())
    }

    /// The first free slot, or `None` when no slot is free.
    pub fn get_next_free(&self) -> Option<ObjectIndex> {
        ObjectIndex::new(self.get__next_free_idx())
    }

    /// Makes `next_free` the first free slot.
    pub fn set_next_free_idx(&mut self, v: Option<ObjectIndex>) {
        self.set__next_free_idx(v.to_bits());
    }
}

impl Default for PartialMeta {
    fn default() -> Self {
        PartialMeta::new()
            .next_free_idx(ObjectIndex::new(0))
            .total_allocated(0)
    }
}

/// The metadata of a free or full slab, its place in its list.
///
/// It stores the bits of an `Option<`[`Prev`]`>`, because a descriptor's
/// metadata must take any bit pattern for the casts between its states.
#[repr(transparent)]
#[derive(Clone, Copy, FromBytes, IntoBytes, Immutable)]
pub struct FullFreeMeta(u32);

impl FullFreeMeta {
    /// Where this slab sits in its list, or `None` in no list.
    pub const fn prev(&self) -> Option<Prev> { Prev::from_bits(self.0) }

    /// Puts this slab at `prev` in its list, or in no list for `None`.
    pub fn set_prev(&mut self, prev: Option<Prev>) {
        self.0 = Prev::to_bits(prev);
    }
}

const impl Default for FullFreeMeta {
    fn default() -> Self { Self(Prev::to_bits(None)) }
}

/// The metadata of a slab whose state isn't known yet.
#[repr(transparent)]
#[derive(Clone, Copy, FromBytes, IntoBytes, Immutable)]
pub struct RawMeta(u32);
