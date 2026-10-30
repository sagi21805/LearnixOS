use core::pattern_type;

/// The index of a slot in a slab. `u16::MAX` is never an index, so
/// `Option<SlotIndex>` uses it for `None` and stays 2 bytes.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SlotIndex(pattern_type!(u16 is 0..=65534));

const _: () = assert!(size_of::<Option<SlotIndex>>() == 2);

impl SlotIndex {
    /// The slot index `idx`, or `None` when `idx` is `u16::MAX`.
    pub const fn new(idx: u16) -> Option<Self> {
        match idx {
            u16::MAX => None,
            // SAFETY: `idx` isn't `u16::MAX`, so it is in the pattern's
            // range.
            idx => Some(Self(unsafe { core::mem::transmute(idx) })),
        }
    }

    /// The index as a plain `u16`.
    pub const fn get(self) -> u16 {
        // SAFETY: every value of the pattern type is a valid `u16`.
        unsafe { core::mem::transmute(self.0) }
    }
}
