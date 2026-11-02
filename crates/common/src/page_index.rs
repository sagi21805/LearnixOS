use core::{fmt, pattern_type};

/// The index of a page in the buddy arena. The two largest `u32`s are
/// never an index, so `Option<PageIndex>` uses one for `None` and
/// `Option<`[`Prev`]`>` uses both, in 4 bytes.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageIndex(Indices);

/// The `u32`s that are page indices.
type Indices = pattern_type!(u32 is 0..=0xFFFF_FFFD);

const _: () = assert!(size_of::<Option<PageIndex>>() == 4);

impl PageIndex {
    /// The page index `idx`, or `None` when `idx` is one of the two
    /// largest `u32`s.
    pub const fn new(idx: u32) -> Option<Self> {
        match idx {
            0xFFFF_FFFE.. => None,
            // SAFETY: `idx` is below `0xFFFF_FFFE`, so it is in the
            // pattern's range.
            idx => Some(Self(unsafe {
                core::mem::transmute::<u32, Indices>(idx)
            })),
        }
    }

    /// The index as a plain `u32`.
    pub const fn get(self) -> u32 {
        // SAFETY: every value of the pattern type is a valid `u32`.
        unsafe { core::mem::transmute::<Indices, u32>(self.0) }
    }
}

impl PartialEq for PageIndex {
    fn eq(&self, other: &Self) -> bool { self.get() == other.get() }
}

impl Eq for PageIndex {}

impl fmt::Debug for PageIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

/// Where a node sits in its list, with `None` for a node in no list.
///
/// Every `u32` is an `Option<Prev>`, because [`After`](Prev::After) takes
/// every index, and [`First`](Prev::First) and `None` take the two `u32`s
/// that aren't one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Prev {
    /// First in its list, which the list's head points at.
    First,
    /// Right after the node at this index.
    Page(PageIndex),
}

const _: () = assert!(size_of::<Option<Prev>>() == 4);

impl Prev {
    /// The place stored in `bits`, or `None` for no list.
    pub const fn from_bits(bits: u32) -> Option<Prev> {
        // SAFETY: every `u32` is an `Option<Prev>`.
        unsafe { core::mem::transmute::<u32, Option<Prev>>(bits) }
    }

    /// The bits that store `prev`.
    pub const fn to_bits(prev: Option<Prev>) -> u32 {
        // SAFETY: an `Option<Prev>` is 4 initialized bytes.
        unsafe { core::mem::transmute::<Option<Prev>, u32>(prev) }
    }
}
