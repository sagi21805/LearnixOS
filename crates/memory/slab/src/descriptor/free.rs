extern crate alloc;

use super::{
    Free, FullFreeMeta, Partial, PartialMeta, SlabAddress, SlabDescriptor,
    SlabStateKind,
};
use crate::{
    descriptor::{FreeDetached, Full, FullDetached, PartialDetached},
    preallocated::PreAllocated,
    traits::{Attach, ConvertInplace, Detach, SelfAttach, Slab},
};
use alloc::alloc::{Layout, alloc};
use common::constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE};
use core::{mem::size_of, ptr::NonNull};
use nonmax::NonMaxU16;

impl<T: Slab> SlabDescriptor<T, FreeDetached> {
    /// Create a new, free slab descriptor.
    pub fn new(order: usize) -> SlabDescriptor<T, FreeDetached> {
        todo!("Use direct call on buddy allocator for certain page size.");
        let address = unsafe {
            NonNull::new_unchecked(alloc(
                Layout::from_size_alignment_unchecked(
                    REGULAR_PAGE_SIZE * (1 << order),
                    REGULAR_PAGE_ALIGNMENT,
                ),
            ))
            .cast::<PreAllocated<T>>()
        };

        let mut objects = NonNull::slice_from_raw_parts(
            address,
            (REGULAR_PAGE_SIZE * (1 << order))
                / size_of::<PreAllocated<T>>(),
        );

        // Initialize each free object to point at the next free.
        for (i, object) in
            unsafe { objects.as_mut() }.iter_mut().enumerate()
        {
            *object = PreAllocated {
                next_free_idx: Some(unsafe {
                    NonMaxU16::new_unchecked(i as u16 + 1)
                }),
            }
        }

        unsafe {
            // Set the last object free_index into none because it is the
            // end of the slab.
            objects.as_mut().last_mut().unwrap().next_free_idx = None;

            SlabDescriptor {
                state: FullFreeMeta::new()
                    .prev(SlabAddress(None))
                    .partial(false),
                objects: objects.cast(),
                next: None,
            }
        }
    }
}

impl<T> SlabDescriptor<T, Free>
where
    T: Slab,
{
    pub fn alloc<'a>(
        &'a mut self,
        head: &'a mut Option<&'a mut SlabDescriptor<T, Partial>>,
    ) -> (NonNull<T>, SlabStateKind) {
        let detached = self.detach();

        let partial = detached.convert_inplace(PartialMeta::default());

        match head {
            Some(head) => {
                let attached = head.attach_partial(partial);
                attached.alloc()
            }
            None => {
                let attached = partial.attach_self();
                let allocation = attached.alloc();
                *head = Some(attached);
                allocation
            }
        }
    }
}

impl<T: Slab> Detach<T, Free> for SlabDescriptor<T, Free> {
    fn detach(&mut self) -> &mut SlabDescriptor<T, FreeDetached> {
        self.detach_linked()
    }
}

impl<T: Slab> Attach<T, Free> for SlabDescriptor<T, Free> {
    fn attach_free(
        &mut self,
        other: &mut SlabDescriptor<T, FreeDetached>,
    ) -> &mut SlabDescriptor<T, Free> {
        self.attach_linked(other)
    }

    fn attach_full(
        &mut self,
        _other: &mut SlabDescriptor<T, FullDetached>,
    ) -> &mut SlabDescriptor<T, Free> {
        unimplemented!()
    }

    fn attach_partial(
        &mut self,
        other: &mut SlabDescriptor<T, PartialDetached>,
    ) -> &mut SlabDescriptor<T, Free> {
        let free = other.convert_inplace(FullFreeMeta::default());
        self.attach_linked(free)
    }
}

// impl<T: Slab> Attach<T> for SlabDescriptor<T, Free> {
//     fn attach(&mut self, other: &mut SlabDescriptor<T, Partial>) {
//         let free = other.convert_inplace(
//             FullFreeMeta::new()
//                 .partial(false)
//
// .prev(SlabAddress::from_non_null(NonNull::from_mut(self))),         );

//         self.attach_linked(free);
//     }
// }

impl<T: Slab> ConvertInplace<T, PartialDetached, FreeDetached>
    for SlabDescriptor<T, FreeDetached>
{
    fn convert_inplace(
        &mut self,
        meta: PartialMeta,
    ) -> &mut SlabDescriptor<T, PartialDetached> {
        debug_assert!(meta == PartialMeta::default());
        let partial = unsafe {
            core::mem::transmute::<
                &mut SlabDescriptor<T, FreeDetached>,
                &mut SlabDescriptor<T, PartialDetached>,
            >(self)
        };
        partial.state = meta;
        partial
    }
}
