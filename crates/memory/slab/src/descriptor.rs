extern crate alloc;

use crate::traits::Slab;
use alloc::alloc::{Layout, alloc};
use common::constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE};
use core::{
    fmt::Debug,
    mem::{ManuallyDrop, size_of},
    ptr::NonNull,
};
use macros::bitfields;
use nonmax::{NonMaxU16, NonMaxU32};

#[derive(Clone, Copy)]
struct Free {
    next_free_idx: Option<NonMaxU32>,
    total_allocated: u32,
}

#[derive(Debug)]
struct SlabAddr(u64);

#[rustfmt::skip]
impl const From<u64> for SlabAddr {
    fn from(value: u64) -> Self {
        SlabAddr(value)
    }
}

#[rustfmt::skip]
impl const From<SlabAddr> for u64 {
    fn from(value: SlabAddr) -> Self {
        value.0
    }
}

impl SlabAddr {
    pub unsafe fn as_ptr<T: Slab>(
        &self,
    ) -> Option<NonNull<SlabDescriptor<T>>> {
        NonNull::new(self.0 as *mut SlabDescriptor<T>)
    }

    pub fn from_ptr<T: Slab>(&mut self, ptr: NonNull<SlabDescriptor<T>>) {
        self.0 = ptr.addr().get() as u64
    }
}

// TODO: Future useful addition could be to remmove the unsafe cast on slab
// addr, and support type T inside the bitfields macro.
#[bitfields]
struct Full {
    #[flag(flag_type = SlabAddr)]
    prev: B63,
    full: B1,
}

pub union SlabState {
    free: Free,
    full: Full,
}

#[repr(C)]
pub struct SlabDescriptor<T: Slab> {
    pub state: SlabState,
    // TODO: Check the possibility to not save the length here because it
    // is already managed by a freelist so the len may not be needed.
    //
    // Moreover the local T holds the const order in which we allocate for
    // the array so the size can always be calculated.
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
