use common::enums::PageSize;
use x86::structures::paging::PageEntryFlags;

/// Get the position on the slab array, for a slab of the given type.
///
/// Shouldn't implement this trait manually; it is implemented
/// via the `define_slab_system` macro.
pub trait Slab: 'static + Sized + SlabPosition + SlabFlags {}

impl Slab for () {}

pub trait SlabPosition {
    const SLAB_POSITION: usize;
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

impl SlabFlags for () {
    const PFLAGS: PageEntryFlags = PageEntryFlags::default();
    const PSIZE: PageSize = PageSize::Regular;
}

pub trait SlabCacheConstructor {
    fn new(buddy_order: usize) -> Self;
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
