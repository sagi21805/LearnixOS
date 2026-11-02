use core::{alloc::Layout, marker::Unsize, mem::Alignment};

use common::enums::PageSize;
use typestate_groups::{Isomorphic, group, state, state_types};
use x86::structures::paging::PageEntryFlags;

use crate::{
    descriptor::{SlabDescriptor, Unused, Used},
    preallocated::PreAllocated,
    object_index::ObjectIndex,
};

/// A type allocated from its own slab cache, the one at `SLAB_POSITION`
/// in [`SlabCaches`](crate::cache::SlabCaches).
pub trait Slab:
    SlabGeometry + SlabFamily + sealed::Sealed + 'static
{
    /// The index of this type's cache, which no other type shares.
    const SLAB_POSITION: usize;
}

/// The type of the caches in [`SlabCaches`](crate::cache::SlabCaches)
/// before one is cast to the slab it holds. It has no cache of its own.
#[state]
pub struct Unknown;

impl Slab for Unknown {
    const SLAB_POSITION: usize = usize::MAX;
}

impl SlabGeometry for Unknown {
    type PreAllocatedArray = [PreAllocated<Unknown>;
        <Unknown as SlabGeometry>::OBJECTS_PER_SLAB];
}

impl sealed::Sealed for Unknown {}

#[group(UnknownSlabs)]
impl SlabFamily for (Unknown,) {
    const PFLAGS: PageEntryFlags = PageEntryFlags::regular_page_flags();
    const PSIZE: PageSize = PageSize::Regular;
}

/// Keeps [`Slab`] to the types `define_slab_system!` numbers, as
/// [`SlabCaches::cache`](crate::cache::SlabCaches::cache) trusts that no
/// two of them share a position.
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The sizes of a slab of `Self`, all derived from the size of its slot
/// and the page size of its family.
pub trait SlabGeometry: SlabFamily + Sized {
    /// The buddy order of one slab, the smallest that fits an object.
    const BUDDY_ORDER: usize = size_of::<PreAllocated<Self>>()
        .div_ceil(Self::PSIZE.size())
        .next_power_of_two()
        .ilog2() as usize;

    const PAGES_PER_SLAB: usize = 1 << Self::BUDDY_ORDER;

    /// The objects in one slab. It fits a `u16`, which slot indices and
    /// allocation counts are.
    const OBJECTS_PER_SLAB: usize = {
        let objects = Self::PAGES_PER_SLAB * Self::PSIZE.size()
            / size_of::<PreAllocated<Self>>();
        assert!(
            objects <= u16::MAX as usize,
            "a slab's objects fit a `u16`"
        );
        objects
    };
    /// All `OBJECT_PER_SLAB` objects of one slab as one array, so a thin
    /// pointer to it covers every object. Only an array unsizes to a
    /// slice.
    type PreAllocatedArray: Unsize<[PreAllocated<Self>]>;
    /// The alignment of every object, the slot's alignment capped at the
    /// alignment of the page the slab starts on.
    const OBJECT_ALIGNMENT: Alignment =
        Alignment::of::<PreAllocated<Self>>().min(Self::PSIZE.alignment());
    /// The layout of one slab's pages. A slab too large for a `Layout`
    /// fails the build.
    const SLAB_LAYOUT: Layout = match Layout::from_size_align(
        Self::PAGES_PER_SLAB * Self::PSIZE.size(),
        Self::PSIZE.size(),
    ) {
        Ok(layout) => layout,
        Err(_) => panic!("a slab's pages fit in a `Layout`"),
    };
    /// The power of two in the slot size, `size = odd << SLOT_SHIFT`.
    const SLOT_SHIFT: u32 =
        size_of::<PreAllocated<Self>>().trailing_zeros();
    /// The inverse of the slot size's odd part modulo 2^64, so a multiple
    /// of the slot size divides by it with one multiply and one shift.
    const SLOT_INVERSE: usize =
        odd_inverse(size_of::<PreAllocated<Self>>() >> Self::SLOT_SHIFT);

    /// The index of the slot that starts `offset` bytes into a slab, or
    /// `None` when no slot starts there.
    ///
    /// It returns `offset / size` when
    /// `offset % size == 0 && offset / size < OBJECT_PER_SLAB`, with one
    /// multiply and one comparison. The slot size splits into an odd part
    /// and a power of two, `size = d << SLOT_SHIFT`, and `offset` goes
    /// through three steps.
    ///
    /// 1. Multiplying by `SLOT_INVERSE`, the inverse of `d` modulo 2^64,
    ///    divides by `d`. Multiplying by an odd number permutes the 64-bit
    ///    values, and it sends each multiple `k * d` to `k`. The multiples
    ///    fill `0..=usize::MAX / d`, so every other offset lands above it.
    /// 2. Rotating right by `SLOT_SHIFT` divides by the power of two. An
    ///    odd multiplier keeps the offset's trailing zeros, so an offset
    ///    that isn't a multiple of `1 << SLOT_SHIFT` rotates set bits into
    ///    the top of the result.
    /// 3. An offset taken from an address before the slab wraps around to
    ///    near `usize::MAX`, and the steps above keep it huge.
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
    fn object_index_at(offset: usize) -> Option<ObjectIndex> {
        let idx = offset
            .wrapping_mul(Self::SLOT_INVERSE)
            .rotate_right(Self::SLOT_SHIFT);

        (idx < Self::OBJECTS_PER_SLAB)
            .then(|| ObjectIndex::new(idx as u16))
            .flatten()
    }

    /// Whether an object of `Self` can hold `layout`.
    fn fits(layout: Layout) -> bool {
        layout.size() <= size_of::<Self>()
            && layout.alignment() <= Self::OBJECT_ALIGNMENT
    }
}

/// The inverse of the odd `d` modulo 2^64, by Newton's iteration.
///
/// If `d * x = 1 - e` with `e` a multiple of 2^k, the step
/// `x' = x * (2 - d * x)` gives `d * x' = 1 - e^2`, and `e^2` is a
/// multiple of 2^2k. Each step doubles the correct low bits. The square
/// of an odd number is 1 modulo 8, so `x = d` starts with 3 correct bits,
/// and five steps reach 3 * 2^5 = 96 bits, more than 64.
const fn odd_inverse(d: usize) -> usize {
    let mut x = d;
    let mut step = 0;
    while step < 5 {
        x = x.wrapping_mul(2usize.wrapping_sub(d.wrapping_mul(x)));
        step += 1;
    }
    assert!(d.wrapping_mul(x) == 1, "`d` is odd");
    x
}

/// The family of a slab type, whose group sets the flags and page size
/// its pages are mapped with.
#[state_types]
pub trait SlabFamily {
    const PFLAGS: PageEntryFlags;
    const PSIZE: PageSize;
}

// TODO: declare a `DmaGeneric` group with the DMA sizes, mapped with
// `regular_io_page_flags()`.

pub trait SlabBlock {
    /// The [`SlabDescriptor`] of the slab this block holds.
    ///
    /// # Safety
    ///
    /// The block must hold a slab of `T`, as the descriptor is cast to
    /// `T` without a check.
    unsafe fn slab_descriptor<T: Slab>(&self) -> &SlabDescriptor<T, Used>;

    /// The [`SlabDescriptor`] of the slab this block holds, mutably.
    ///
    /// The descriptor sits outside what a shared reference to the block
    /// freezes, so buddy can read the block's flags meanwhile.
    ///
    /// # Safety
    ///
    /// The caller must own the block as a slab of `T`, as the descriptor
    /// is cast to `T` without a check, and must hold no other reference
    /// to the descriptor.
    #[expect(
        clippy::mut_from_ref,
        reason = "the descriptor sits in an `UnsafeCell`, and only the \
                  block's owner reaches it"
    )]
    unsafe fn slab_descriptor_mut<T: Slab>(
        &self,
    ) -> &mut SlabDescriptor<T, Used>;

    /// The descriptor storage of this block, ready for
    /// [`SlabDescriptor::assign`] to write a new descriptor into.
    ///
    /// # Safety
    ///
    /// The caller must own the block and hold no slab in use in it, so no
    /// list or reference reaches its descriptor.
    #[expect(
        clippy::mut_from_ref,
        reason = "the descriptor sits in an `UnsafeCell`, and only the \
                  block's owner reaches it"
    )]
    unsafe fn unused_slab_descriptor_mut(
        &self,
    ) -> &mut SlabDescriptor<Unknown, Unused> {
        // SAFETY: any block's descriptor storage reads as `Unknown`.
        unsafe { self.slab_descriptor_mut::<Unknown>() }
            .cast_state_mut::<Unused>()
    }
}
