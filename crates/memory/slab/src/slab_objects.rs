use core::ptr::NonNull;

use common::address_types::VirtualAddress;
use nonmax::NonMaxU16;

use crate::{preallocated::PreAllocated, traits::Slab};

/// The object slots of one slab, in pages that no other value manages.
///
/// It is neither `Copy` nor `Clone` and its pointer is private, so each
/// slab's pages have exactly one manager.
#[repr(transparent)]
pub struct SlabObjects<T: Slab>(NonNull<T::PreAllocatedArray>);

// SAFETY: only this value reaches its free slots, and no method reads the
// `T` in an allocated one.
unsafe impl<T: Slab> Send for SlabObjects<T> {}

impl<T: Slab> SlabObjects<T> {
    /// Takes over the `T::PAGES_PER_SLAB` pages at `pages`.
    ///
    /// # Safety
    ///
    /// `pages` must start `T::PAGES_PER_SLAB` live pages that nothing else
    /// points at.
    pub fn new(pages: NonNull<u8>) -> Self {
        debug_assert!(
            pages.is_aligned_to(T::PSIZE.alignment().as_usize()),
            "slab pages are page-aligned"
        );
        Self(pages.cast())
    }

    /// Where the slab's pages start.
    pub fn address(&self) -> VirtualAddress {
        VirtualAddress::from(self.0.addr().get())
    }

    /// The slot at `idx`.
    ///
    /// # Safety
    ///
    /// `idx` must be less than `T::OBJECT_PER_SLAB`, which debug builds
    /// check.
    pub unsafe fn slot(&self, idx: usize) -> NonNull<PreAllocated<T>> {
        // SAFETY: the caller keeps `idx` inside the slab.
        unsafe { self.objects().get_unchecked_mut(idx) }
    }

    /// The index of `object`, or `None` when it isn't a slot of this
    /// slab.
    ///
    /// The slot size splits into an odd part and a power of two,
    /// `size = d << SLOT_SHIFT`, and the offset of `object` from the slab
    /// start goes through three steps.
    ///
    /// 1. Multiplying by `SLOT_INVERSE`, the inverse of `d` modulo 2^64,
    ///    divides by `d`. Multiplying by an odd number permutes the 64-bit
    ///    values, and it sends each multiple `k * d` to `k`. The multiples
    ///    fill `0..=usize::MAX / d`, so every other offset lands above it.
    /// 2. Rotating right by `SLOT_SHIFT` divides by the power of two. An
    ///    odd multiplier keeps the offset's trailing zeros, so an offset
    ///    that isn't a multiple of `1 << SLOT_SHIFT` rotates set bits into
    ///    the top of the result.
    /// 3. An offset before the slab wraps around to near `usize::MAX`, and
    ///    the steps above keep it huge.
    ///
    /// Each bad offset ends far above `OBJECT_PER_SLAB`, so one comparison
    /// rejects all of them. With 48-byte slots, `d = 3`, `SLOT_SHIFT = 4`
    /// and `SLOT_INVERSE = 0xAAAA_AAAA_AAAA_AAAB`.
    ///
    /// | offset | after the rotate        | result              |
    /// |--------|-------------------------|---------------------|
    /// | 96     | `0x2`                   | slot 2              |
    /// | 100    | `0xCAAA_AAAA_AAAA_AAAC` | not 16-aligned      |
    /// | 112    | `0x0AAA_AAAA_AAAA_AAAD` | not a multiple of 3 |
    /// | -48    | `0x0FFF_FFFF_FFFF_FFFF` | before the slab     |
    pub fn index_of(&self, object: NonNull<T>) -> Option<NonMaxU16> {
        let idx = object
            .addr()
            .get()
            .wrapping_sub(self.0.addr().get())
            .wrapping_mul(T::SLOT_INVERSE)
            .rotate_right(T::SLOT_SHIFT);

        (idx < T::OBJECT_PER_SLAB)
            .then(|| NonMaxU16::new(idx as u16))
            .flatten()
    }

    /// Links every slot to the one after it, and the last to none.
    ///
    /// # Safety
    ///
    /// No slot may be allocated.
    pub unsafe fn link_all(&mut self) {
        // SAFETY: no slot is allocated, so nothing else references them.
        let slots = unsafe { self.objects().as_mut() };

        for (slot, next) in slots.iter_mut().zip(1..) {
            slot.next_free_idx = NonMaxU16::new(next);
        }
        if let Some(last) = slots.last_mut() {
            last.next_free_idx = None;
        }
    }

    /// Every object of the slab, as a slice pointer whose length comes
    /// from `T::PreAllocatedArray`.
    fn objects(&self) -> NonNull<[PreAllocated<T>]> { self.0 }
}
