extern crate alloc;

use crate::traits::Slab;
use alloc::alloc::{Layout, alloc};
use common::constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE};
use core::{
    alloc::LayoutError,
    fmt::Debug,
    mem::{ManuallyDrop, size_of},
    ptr::NonNull,
};
use nonmax::NonMaxU16;

/// Preallocated object in the slab allocator.
pub union PreAllocated<T: Sized> {
    pub allocated: ManuallyDrop<T>,
    pub next_free_idx: Option<NonMaxU16>,
}

impl<T: Debug> Debug for PreAllocated<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PreAllocated")
            .field("allocated", unsafe { &self.allocated })
            .field("next_free_idx", unsafe { &self.next_free_idx })
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SlabDescriptor<T: Slab> {
    pub next_free_idx: Option<NonMaxU16>,
    pub total_allocated: u16,
    // TODO: Check the possibility to not save the length here because it
    // is already managed by a freelist so the len may not be needed.
    pub objects: NonNull<[PreAllocated<T>]>,
    pub next: Option<NonNull<SlabDescriptor<T>>>,
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

        // Set the last object free_index into none because it is the end
        // of the slab.
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

        todo!(
            "Didn't handle the case the slab is full becasue of unwrap. \
             Should allocate another slab"
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
        todo!(
            "Should think if calling drop is the responisibility of the \
             allocator"
        );
        todo!(
            "Should add a check if the ptr that is freed from this slab \
             is actually allocated from it "
        );

        let freed_index = (ptr.as_ptr().addr()
            - self.objects.as_ptr().addr())
            / size_of::<PreAllocated<T>>();

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
