use super::{WORDS, slot::Slot};

/// The free slots of the task table, one bit each, set while free.
pub struct FreeSlots([u64; WORDS]);

impl FreeSlots {
    /// Every slot free.
    pub const fn new() -> Self { Self([u64::MAX; WORDS]) }

    /// The lowest free slot, which is no longer free.
    pub fn take(&mut self) -> Option<Slot> {
        let (word, bits) = self
            .0
            .iter_mut()
            .enumerate()
            .find(|(_, bits)| **bits != 0)?;
        let bit = bits.trailing_zeros() as usize;
        *bits &= *bits - 1; // clears the lowest set bit
        Some(Slot::new(word * u64::BITS as usize + bit))
    }

    /// Frees `slot`.
    pub fn put(&mut self, slot: Slot) {
        debug_assert!(
            self.0[slot.word()] & slot.bit() == 0,
            "only a taken slot is freed"
        );
        self.0[slot.word()] |= slot.bit();
    }
}
