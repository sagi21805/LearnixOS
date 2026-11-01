use core::{fmt, pattern_type};

/// The index of a page in the buddy arena. The two largest `u32`s are
/// never an index, so `Option<PageIndex>` uses one for `None` and an enum
/// with two unit variants and one index variant uses both, in 4 bytes.
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
