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

/// Get the position on the slab array, for a slab of the given type.
///
/// Shouldn't implement this trait manually; it is implemented
/// via the `define_slab_system` macro.
pub trait Slab: SlabPosition + SlabFlags + 'static {}

impl Slab for () {}

pub trait SlabPosition: Sized {
    const SLAB_POSITION: usize;
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

impl SlabPosition for () {
    const SLAB_POSITION: usize = usize::MAX;
}

pub trait SlabFlags: SlabPosition {
    const PFLAGS: PageEntryFlags;
    const PSIZE: PageSize;
}

impl<T: SlabPosition> SlabFlags for T {
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
