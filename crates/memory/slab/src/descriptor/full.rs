use super::{Full, SlabDescriptor};
use crate::{
    descriptor::{
        FreeDetached, FullDetached, FullFreeMeta, PartialDetached,
        PartialMeta,
    },
    traits::{Attach, ConvertInplace, Detach, Slab},
};

impl<T: Slab> Attach<T> for SlabDescriptor<T, Full> {
    fn attach_free(
        &mut self,
        _other: &mut SlabDescriptor<T, FreeDetached>,
    ) {
        unimplemented!()
    }
    fn attach_full(
        &mut self,
        other: &mut SlabDescriptor<T, FullDetached>,
    ) {
        self.attach_linked(other);
    }
    fn attach_partial(
        &mut self,
        other: &mut SlabDescriptor<T, PartialDetached>,
    ) {
        let full =
            other.convert_inplace(FullFreeMeta::new().partial(false));
        self.attach_linked(full);
    }
}

impl<T: Slab> Detach<T, Full> for SlabDescriptor<T, Full> {
    fn detach(&mut self) -> &mut SlabDescriptor<T, FullDetached> {
        self.detach_linked()
    }
}

impl<T: Slab> ConvertInplace<T, PartialDetached, FullDetached>
    for SlabDescriptor<T, FullDetached>
{
    fn convert_inplace(
        &mut self,
        meta: PartialMeta,
    ) -> &mut SlabDescriptor<T, PartialDetached> {
        let partial = unsafe {
            core::mem::transmute::<
                &mut SlabDescriptor<T, FullDetached>,
                &mut SlabDescriptor<T, PartialDetached>,
            >(self)
        };
        partial.state = meta;
        partial
    }
}
