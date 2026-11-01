use crate::{
    descriptor::{SlabDescriptor, SlabState},
    slab_objects::SlabObjects,
    traits::Slab,
};

/// The arena that holds every slab's descriptor, which finds a slab's
/// pages from its descriptor.
pub trait SlabArena {
    /// The slots of the slab whose descriptor is `descriptor`.
    fn objects_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> SlabObjects<T>;
}
