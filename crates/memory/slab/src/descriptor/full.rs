use crate::{
    descriptor::{
        FreeDetached, Full, FullDetached, PartialDetached, PartialMeta,
        SlabAddress, SlabDescriptor, meta::FullFreeMeta,
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
        let full = other.convert_inplace(
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

impl<T: Slab> ConvertInplace<T, PartialDetached, FullDetached>
    for SlabDescriptor<T, FullDetached>
{
    fn convert_inplace(
        &mut self,
        meta: PartialMeta,
    ) -> &mut SlabDescriptor<T, PartialDetached> {
        debug_assert!(meta.is_partial());
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
