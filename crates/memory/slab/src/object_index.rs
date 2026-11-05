use core::pattern_type;

use extend::ext;

/// The index of an object in a slab. `u16::MAX` is never an index, so
/// `Option<ObjectIndex>` uses it for `None` and stays 2 bytes, which makes
/// every `u16` an `Option<ObjectIndex>`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ObjectIndex(pattern_type!(u16 is 0..=65534));

const _: () = assert!(size_of::<Option<ObjectIndex>>() == 2);

impl ObjectIndex {
    /// The object index `idx`, or `None` when `idx` is `u16::MAX`.
    pub const fn new(idx: u16) -> Option<Self> {
        match idx {
            u16::MAX => None,
            // SAFETY: `idx` isn't `u16::MAX`, so it is in the pattern's
            // range.
            idx => Some(Self(unsafe {
                core::mem::transmute::<u16, pattern_type!(u16 is 0..=65534)>(
                    idx,
                )
            })),
        }
    }

    /// The index as a plain `u16`.
    pub const fn get(self) -> u16 {
        // SAFETY: every value of the pattern type is a valid `u16`.
        unsafe { core::mem::transmute(self.0) }
    }
}

/// The bits of an object index, or of no object.
#[ext(name = ObjectIndexBits)]
pub impl Option<ObjectIndex> {
    /// The bits that store this index, or no index.
    fn to_bits(self) -> u16 {
        // SAFETY: an `Option<ObjectIndex>` is 2 initialized bytes.
        unsafe { core::mem::transmute::<Option<ObjectIndex>, u16>(self) }
    }
}
