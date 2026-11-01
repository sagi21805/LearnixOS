use bitfields::bitfields;
use common::page_index::PageIndex;
use zerocopy::{FromBytes, Immutable, IntoBytes};

#[bitfields]
#[derive(PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
pub struct PartialMeta {
    /// The first free slot, or `u16::MAX` when no slot is free.
    pub next_free_idx: B16,
    pub total_allocated: B16,
}

const impl Default for PartialMeta {
    fn default() -> Self {
        PartialMeta::new().next_free_idx(0).total_allocated(0)
    }
}

/// The metadata of a free or full slab, the previous node of its list.
///
/// It stores the raw index, with `u32::MAX` for no node, because a
/// descriptor's metadata must take any bit pattern for the casts between
/// its states.
#[repr(transparent)]
#[derive(Clone, Copy, FromBytes, IntoBytes, Immutable)]
pub struct FullFreeMeta(u32);

impl FullFreeMeta {
    /// The previous node, or `None` at the head of the list.
    pub const fn prev(&self) -> Option<PageIndex> {
        PageIndex::new(self.0)
    }

    /// Makes `prev` the previous node.
    pub fn set_prev(&mut self, prev: Option<PageIndex>) {
        self.0 = prev.map_or(u32::MAX, PageIndex::get);
    }
}

const impl Default for FullFreeMeta {
    fn default() -> Self { Self(u32::MAX) }
}

/// The metadata of a slab whose state isn't known yet.
#[repr(transparent)]
#[derive(Clone, Copy, FromBytes, IntoBytes, Immutable)]
pub struct RawMeta(u32);
