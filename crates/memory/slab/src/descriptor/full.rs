use crate::{
    descriptor::{
        FullDetached, PartialDetached, SlabDescriptor, meta::FullFreeMeta,
    },
    slab_address::SlabAddress,
    traits::Slab,
};
use typestate_groups::MorphFrom;

/// A partial slab allocating its last object becomes full, outside any
/// list.
impl<T: Slab> MorphFrom<SlabDescriptor<T, PartialDetached>>
    for SlabDescriptor<T, FullDetached>
{
    fn morph_from(src: SlabDescriptor<T, PartialDetached>) -> Self {
        debug_assert_eq!(
            src.state.get_total_allocated(),
            T::OBJECT_PER_SLAB as u32
        );

        SlabDescriptor {
            state: FullFreeMeta::default(),
            objects: src.objects,
            next: SlabAddress::default(),
        }
    }
}
