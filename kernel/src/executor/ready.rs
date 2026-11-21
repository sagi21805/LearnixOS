use core::sync::atomic::{AtomicU64, Ordering};

use super::{WORDS, slot::Slot};

/// The slots whose tasks wait for a poll, one bit each.
pub static READY: ReadySet = ReadySet::new();

/// One bit per slot, set while the slot's task waits for a poll.
///
/// Wakers set bits, often inside an interrupt handler, and only the
/// executor clears them.
pub struct ReadySet([AtomicU64; WORDS]);

impl ReadySet {
    pub const fn new() -> Self {
        Self([const { AtomicU64::new(0) }; WORDS])
    }

    /// Marks `slot` ready. One `lock or`, so it is safe in a handler.
    ///
    /// `Release` publishes what the waker wrote before it, such as a scan
    /// code, to the poll that [`take`](Self::take) leads to.
    pub fn wake(&self, slot: Slot) {
        self.0[slot.word()].fetch_or(slot.bit(), Ordering::Release);
    }

    /// Takes every ready slot, clearing their bits one word at a time.
    ///
    /// Each word is cleared before its slots are polled, so a wake during
    /// a poll sets its bit again and the next `take` returns the slot.
    pub fn take(&self) -> impl Iterator<Item = Slot> + '_ {
        (0..WORDS).flat_map(|word| ReadySlots {
            word,
            bits: self.0[word].swap(0, Ordering::Acquire),
        })
    }

    /// Whether no slot is ready.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|word| word.load(Ordering::Relaxed) == 0)
    }
}

impl Default for ReadySet {
    fn default() -> Self { Self::new() }
}

/// The slots one word's swap found ready, lowest first.
struct ReadySlots {
    word: usize,
    bits: u64,
}

impl Iterator for ReadySlots {
    type Item = Slot;

    fn next(&mut self) -> Option<Slot> {
        if self.bits == 0 {
            return None;
        }
        let bit = self.bits.trailing_zeros() as usize;
        self.bits &= self.bits - 1; // clears the lowest set bit
        Some(Slot::new(self.word * u64::BITS as usize + bit))
    }
}
