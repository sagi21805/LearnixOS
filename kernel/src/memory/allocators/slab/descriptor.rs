use super::traits::SlabPosition;
use crate::{alloc_pages, memory::page_descriptor::Unassigned};
use common::constants::REGULAR_PAGE_SIZE;
use core::{
    fmt::Debug,
    mem::{ManuallyDrop, size_of},
    ptr::NonNull,
};
use nonmax::NonMaxU16;

/// Preallocated object in the slab allocator.
pub union PreallocatedObject<T: 'static + Sized> {
    pub allocated: ManuallyDrop<T>,
    pub next_free_idx: Option<NonMaxU16>,
}

impl<T> Debug for PreallocatedObject<T> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SlabDescriptor<T: 'static + Sized + SlabPosition> {
    pub next_free_idx: Option<NonMaxU16>,
    pub objects: NonNull<[PreallocatedObject<T>]>,
    pub next: Option<NonNull<SlabDescriptor<T>>>,
}

impl<T: SlabPosition> SlabDescriptor<T> {
    pub fn new(
        order: usize,
        next: Option<NonNull<SlabDescriptor<T>>>,
    ) -> SlabDescriptor<T> {
        let address = unsafe { alloc_pages!(1 << order).translate() };
        let mut objects = unsafe {
            NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(
                    address.as_mut_ptr::<PreallocatedObject<T>>(),
                ),
                ((1 << order) * REGULAR_PAGE_SIZE)
                    / size_of::<PreallocatedObject<T>>(),
            )
        };

        for (i, object) in
            unsafe { objects.as_mut() }.iter_mut().enumerate()
        {
            *object = PreallocatedObject {
                next_free_idx: Some(unsafe {
                    NonMaxU16::new_unchecked(i as u16 + 1)
                }),
            }
        }

        unsafe {
            objects.as_mut().last_mut().unwrap().next_free_idx = None
        };

        SlabDescriptor {
            next_free_idx: Some(unsafe { NonMaxU16::new_unchecked(0) }),
            objects,
            next,
        }
    }

    pub fn alloc(&mut self) -> NonNull<T> {
        debug_assert!(
            self.next_free_idx.is_some(),
            "Called allocate on a full slab"
        );

        let idx = self.next_free_idx.unwrap().get() as usize;
        let preallocated = unsafe { &mut self.objects.as_mut()[idx] };

        self.next_free_idx = unsafe { preallocated.next_free_idx };

        unsafe { NonNull::from_mut(&mut preallocated.allocated) }
    }

    pub unsafe fn dealloc(&mut self, ptr: *const T) {
        let freed_index = (ptr.addr() - self.objects.as_ptr().addr())
            / size_of::<PreallocatedObject<T>>();

        unsafe {
            self.objects.as_mut()[freed_index].next_free_idx =
                self.next_free_idx;
        };
        self.next_free_idx =
            unsafe { Some(NonMaxU16::new_unchecked(freed_index as u16)) };
    }

    pub fn as_unassigned(&self) -> &SlabDescriptor<Unassigned> {
        unsafe {
            &*(self as *const _ as *const SlabDescriptor<Unassigned>)
        }
    }

    pub fn as_unassigned_mut(
        &mut self,
    ) -> &mut SlabDescriptor<Unassigned> {
        unsafe {
            &mut *(self as *mut _ as *mut SlabDescriptor<Unassigned>)
        }
    }
}

impl SlabDescriptor<Unassigned> {
    pub fn assign<T: SlabPosition>(&self) -> NonNull<SlabDescriptor<T>> {
        unsafe {
            NonNull::new_unchecked(
                self as *const _ as *mut SlabDescriptor<T>,
            )
        }
    }
}

impl SlabDescriptor<SlabDescriptor<Unassigned>> {
    pub fn initial_descriptor(
        order: usize,
    ) -> NonNull<SlabDescriptor<SlabDescriptor<Unassigned>>> {
        let mut descriptor =
            SlabDescriptor::<SlabDescriptor<Unassigned>>::new(order, None);

        let mut ptr = descriptor.alloc();

        unsafe { *ptr.as_mut() = descriptor.as_unassigned().clone() }

        unsafe { ptr.as_ref().assign::<SlabDescriptor<Unassigned>>() }
    }
}
