use crate::{
    alloc_pages,
    memory::{
        allocators::slab::traits::Slab,
        unassigned::{AssignSlab, UnassignSlab},
    },
};
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
pub struct SlabDescriptor<T: Slab> {
    pub next_free_idx: Option<NonMaxU16>,
    pub total_allocated: u16,
    pub objects: NonNull<[PreallocatedObject<T>]>,
    pub next: Option<NonNull<SlabDescriptor<T>>>,
}

impl AssignSlab for NonNull<SlabDescriptor<()>> {
    type Target<Unassigned: Slab> = NonNull<SlabDescriptor<Unassigned>>;

    fn assign<T: Slab>(&self) -> NonNull<SlabDescriptor<T>> {
        unsafe {
            NonNull::new_unchecked(self.as_ptr() as *mut SlabDescriptor<T>)
        }
    }
}

impl<T: Slab> UnassignSlab for NonNull<SlabDescriptor<T>> {
    type Target = NonNull<SlabDescriptor<()>>;

    fn as_unassigned(&self) -> Self::Target {
        unsafe {
            NonNull::new_unchecked(self.as_ptr() as *mut SlabDescriptor<()>)
        }
    }
}

impl<T: Slab> SlabDescriptor<T> {
    /// Create a new slab descriptor.
    ///
    /// # Safety
    /// This function is marked as unsafe because it does not initialize
    /// the page that the allocation is on.
    ///
    /// This function is meant to be called from the [`grow`]
    /// function inside slab cache. (Which is safe and do initialize
    /// the page)
    pub unsafe fn new(
        order: usize,
        next: Option<NonNull<SlabDescriptor<T>>>,
    ) -> SlabDescriptor<T> {
        let address = unsafe { alloc_pages!(1 << order).translate() };

        let mut objects = NonNull::slice_from_raw_parts(
            address.as_non_null::<PreallocatedObject<T>>(),
            ((1 << order) * REGULAR_PAGE_SIZE)
                / size_of::<PreallocatedObject<T>>(),
        );

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
            total_allocated: 0,
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

        self.total_allocated += 1;

        unsafe { NonNull::from_mut(&mut preallocated.allocated) }
    }

    // TODO: In tests rembmber to implement something on T that implement
    // drop and see that when freeing the memory it is called
    pub unsafe fn dealloc(&mut self, ptr: NonNull<T>) {
        todo!("Remember to call drop on the item");

        let freed_index = (ptr.as_ptr().addr()
            - self.objects.as_ptr().addr())
            / size_of::<PreallocatedObject<T>>();

        unsafe {
            self.objects.as_mut()[freed_index].next_free_idx =
                self.next_free_idx;
        };
        self.next_free_idx =
            unsafe { Some(NonMaxU16::new_unchecked(freed_index as u16)) };

        self.total_allocated -= 1;
    }
}

impl SlabDescriptor<SlabDescriptor<()>> {
    /// Return a pointer to the initial descriptor after it allocated
    /// himself.
    ///
    /// The pointer the is returned by this function contains an already
    /// initialized descriptor that allocates itself.
    pub fn initial_descriptor(
        order: usize,
    ) -> NonNull<SlabDescriptor<SlabDescriptor<()>>> {
        let mut descriptor = unsafe {
            SlabDescriptor::<SlabDescriptor<()>>::new(order, None)
        };

        let mut self_allocation = descriptor.alloc();

        unsafe {
            *self_allocation.as_mut() = NonNull::from_ref(&descriptor)
                .as_unassigned()
                .as_ref()
                .clone()
        }

        self_allocation.assign::<SlabDescriptor<()>>()
    }
}
