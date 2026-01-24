use crate::memory::unassigned::Unassigned;

/// Get the position on the slab array, for a slab of the given type.
///
/// Shouldn't implement this trait manually; it is implemented
/// via the `define_slab_system` macro.
pub trait SlabPosition: 'static + Sized {
    const POSITION: usize;
}

impl SlabPosition for Unassigned {
    const POSITION: usize = usize::MAX;
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
