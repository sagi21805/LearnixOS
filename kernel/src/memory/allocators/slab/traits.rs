use crate::memory::page_descriptor::Unassigned;

/// Get the position on the slab array, for a slab of the given type.
///
/// Shouldn't implement this trait manually; it is implemented
/// via the `define_slab_system` macro.
pub trait SlabPosition {
    const POSITION: usize;
}

impl SlabPosition for Unassigned {
    const POSITION: usize = usize::MAX;
}

pub trait SlabCacheConstructor {
    fn new(buddy_order: usize) -> Self;
}
