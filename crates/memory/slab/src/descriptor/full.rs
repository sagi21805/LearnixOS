use crate::{
    descriptor::{
        FreeDetached, Full, FullDetached, PartialDetached, SlabAddress,
        SlabDescriptor, meta::FullFreeMeta,
    },
    traits::{Attach, ConvertInplace, Detach, Slab},
};

impl<T: Slab> Attach<T, Full> for SlabDescriptor<T, Full> {
    fn attach_free(
        &mut self,
        _other: &mut SlabDescriptor<T, FreeDetached>,
    ) -> &mut SlabDescriptor<T, Full> {
        unimplemented!()
    }
    fn attach_full(
        &mut self,
        other: &mut SlabDescriptor<T, FullDetached>,
    ) -> &mut SlabDescriptor<T, Full> {
        self.attach_linked(other)
    }
    fn attach_partial(
        &mut self,
        other: &mut SlabDescriptor<T, PartialDetached>,
    ) -> &mut SlabDescriptor<T, Full> {
        let full = other.convert_to(
            FullFreeMeta::new()
                .partial(false)
                .prev(SlabAddress::default()),
        );
        self.attach_linked(full)
    }
}

impl<T: Slab> Detach<T, Full> for SlabDescriptor<T, Full> {
    fn detach(&mut self) -> &mut SlabDescriptor<T, FullDetached> {
        self.detach_linked()
    }
}
