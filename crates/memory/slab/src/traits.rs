use core::{alloc::Layout, mem::Alignment};

use common::{address_types::VirtualAddress, enums::PageSize};
use typestate_groups::{group, state, state_types};
use x86::structures::paging::PageEntryFlags;

use crate::{
    descriptor::{SlabDescriptor, Used},
    preallocated::PreAllocated,
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

impl SlabGeometry for Unknown {}

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

    const OBJECT_PER_SLAB: usize = Self::PAGES_PER_SLAB
        * Self::PSIZE.size()
        / size_of::<PreAllocated<Self>>();
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
    fn from_address(address: VirtualAddress) -> Self;

    /// The [`SlabDescriptor`] of the slab this block holds.
    ///
    /// # Safety
    ///
    /// The block must hold a slab of `T`, as the descriptor is cast to
    /// `T` without a check.
    unsafe fn slab_descriptor<T: Slab>(&self) -> &SlabDescriptor<T, Used>;

    /// The [`SlabDescriptor`] of the slab this block holds, mutably.
    ///
    /// # Safety
    ///
    /// The block must hold a slab of `T`, as the descriptor is cast to
    /// `T` without a check.
    unsafe fn slab_descriptor_mut<T: Slab>(
        &mut self,
    ) -> &mut SlabDescriptor<T, Used>;
}
