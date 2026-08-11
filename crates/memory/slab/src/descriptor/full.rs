use super::{Full, SlabDescriptor};
use crate::{
    descriptor::FullDetached,
    traits::{Attach, Detach, Slab},
};

impl<T: Slab> Attach<T> for SlabDescriptor<T, Full> {
    fn attach_free(
        &mut self,
        other: &mut SlabDescriptor<T, super::FreeDetached>,
    ) {
    }
    fn attach_full(
        &mut self,
        other: &mut SlabDescriptor<T, super::FullDetached>,
    ) {
    }
    fn attach_partial(
        &mut self,
        other: &mut SlabDescriptor<T, super::PartialDetached>,
    ) {
    }
}

impl<T: Slab> Detach<T, Full> for SlabDescriptor<T, Full> {
    fn detach(&mut self) -> &mut SlabDescriptor<T, FullDetached> {
        todo!()
    }
}
