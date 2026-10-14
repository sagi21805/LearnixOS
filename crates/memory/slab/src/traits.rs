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

    fn size(&self) -> usize;
}

pub trait DmaGeneric {
    const START: usize;
    const END: usize;

    fn size(&self) -> usize;
}

pub trait SlabBlock {
    fn from_address(address: VirtualAddress) -> Self;

    /// Retrive a reference to the [`SlabDescriptor`] from the block.
    ///
    /// # Safety
    ///
    /// The `T` that this function gets will cast the [`SlabDescriptor`]
    /// that in this block unconditionality which in unsafe.
    fn slab_descriptor<T: Slab>(&self) -> &SlabDescriptor<T, Used>;

    /// Retrive a mutable reference [`SlabDescriptor`] from the block.
    ///
    /// # Safety
    ///
    /// The `T` that this function gets will cast the [`SlabDescriptor`]
    /// that in this block unconditionality which in unsafe.
    fn slab_descriptor_mut<T: Slab>(
        &mut self,
    ) -> &mut SlabDescriptor<T, Used>;
}
