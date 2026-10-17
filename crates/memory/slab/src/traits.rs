use core::alloc::Layout;

use common::{
    address_types::VirtualAddress, constants::REGULAR_PAGE_SIZE,
    enums::PageSize,
};
use x86::structures::paging::PageEntryFlags;

use crate::{
    descriptor::{SlabDescriptor, Used},
    preallocated::PreAllocated,
};

/// A type allocated from its own slab cache, the one at `SLAB_POSITION`
/// in [`SlabCaches`](crate::cache::SlabCaches).
pub trait Slab:
    SlabGeometry + SlabFlags + sealed::Sealed + 'static
{
    /// The index of this type's cache, which no other type shares.
    const SLAB_POSITION: usize;
}

/// The erased type of the caches in
/// [`SlabCaches`](crate::cache::SlabCaches), which has no cache of its
/// own.
impl Slab for () {
    const SLAB_POSITION: usize = usize::MAX;
}

impl SlabGeometry for () {}

impl sealed::Sealed for () {}

/// Keeps [`Slab`] to the types `define_slab_system!` numbers, as
/// [`SlabCaches::cache`](crate::cache::SlabCaches::cache) trusts that no
/// two of them share a position.
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The sizes of a slab of `Self`, all derived from the size of its slot.
pub trait SlabGeometry: Sized {
    /// The buddy order of one slab, the smallest that fits an object.
    const BUDDY_ORDER: usize = size_of::<PreAllocated<Self>>()
        .div_ceil(REGULAR_PAGE_SIZE)
        .next_power_of_two()
        .ilog2() as usize;
    const PAGES_PER_SLAB: usize = 1 << Self::BUDDY_ORDER;
    const OBJECT_PER_SLAB: usize = Self::PAGES_PER_SLAB
        * REGULAR_PAGE_SIZE
        / size_of::<PreAllocated<Self>>();
    /// The alignment of every object, the largest power of two that
    /// divides both the slot size and the page size the slab starts on.
    /// As the page size is a power of two, that is the lowest set bit of
    /// their OR.
    const OBJECT_ALIGNMENT: usize = (size_of::<PreAllocated<Self>>()
        | REGULAR_PAGE_SIZE)
        .isolate_lowest_one();
    /// The layout of one slab's pages. A slab too large for a `Layout`
    /// fails the build.
    const SLAB_LAYOUT: Layout = match Layout::from_size_align(
        Self::PAGES_PER_SLAB * REGULAR_PAGE_SIZE,
        REGULAR_PAGE_SIZE,
    ) {
        Ok(layout) => layout,
        Err(_) => panic!("a slab's pages fit in a `Layout`"),
    };
}

pub trait SlabFlags: SlabGeometry {
    const PFLAGS: PageEntryFlags;
    const PSIZE: PageSize;
}

impl<T: SlabGeometry> SlabFlags for T {
    default const PFLAGS: PageEntryFlags =
        PageEntryFlags::regular_page_flags();

    default const PSIZE: PageSize = PageSize::Regular;
}

pub trait Generic {
    const START: usize;
    const END: usize;
}

pub trait DmaGeneric {
    const START: usize;
    const END: usize;

    fn size(&self) -> usize;
}

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
