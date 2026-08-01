extern crate alloc;

use crate::{cache::SlabCache, preallocated::PreAllocated, traits::Slab};
use alloc::alloc::{Layout, alloc};
use common::constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE};
use core::{mem::size_of, ptr::NonNull};
use nonmax::NonMaxU16;

#[repr(C)]
pub struct SlabDescriptor<T, S>
where
    T: Slab,
    S: SlabState<T>,
{
    pub state: S::State,

    // TODO: Check the possibility to not save the length here because it
    // is already managed by a freelist so the len may not be needed.
    //
    // Moreover the local T holds the const order in which we allocate for
    // the array so the size can always be calculated.
    pub objects: NonNull<[PreAllocated<T>]>,
    pub next: Option<NonNull<SlabDescriptor<T, S>>>,
}

// TODO: Seal trait
pub trait SlabState<T: Slab> {
    type State: Sized;
}

pub struct Partial;
impl<T: Slab> SlabState<T> for Partial {
    type State = PartialMeta;
}
pub struct Free;
impl<T: Slab> SlabState<T> for Free {
    type State = Option<NonNull<SlabDescriptor<T, Free>>>;
}
pub struct Full;
impl<T: Slab> SlabState<T> for Full {
    type State = Option<NonNull<SlabDescriptor<T, Full>>>;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PartialMeta {
    pub next_free_idx: Option<NonMaxU16>,
    pub total_allocated: u32,
}

impl<T: Slab> SlabDescriptor<T, Free> {
    /// Create a new, free slab descriptor.
    pub fn new(
        order: usize,
        next: Option<NonNull<SlabDescriptor<T, Free>>>,
    ) -> SlabDescriptor<T, Free> {
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
                state: None,
                objects,
                next,
            }
        }
    }
}

impl<T: Slab> SlabDescriptor<T, Partial> {
    /// Allocate an object from this slab.
    ///
    /// # Parameters
    ///
    /// * `head` - The head of the full slab descriptor. This slab will be
    ///   attached to it, if the allocation will make this slab full.
    pub fn alloc(
        &mut self,
        head: &mut SlabDescriptor<T, Full>,
    ) -> NonNull<T> {
        debug_assert!(
            self.state.is_partial().unwrap().next_free_idx.is_some(),
            "Called allocate on a full slab"
        );

        match self.state.is_partial_mut() {
            Ok(partial) => {
                let idx = partial.next_free_idx.unwrap().get() as usize;
                let preallocated =
                    unsafe { &mut self.objects.as_mut()[idx] };

                partial.next_free_idx =
                    unsafe { preallocated.next_free_idx };

                partial.flags.set_total_allocated(
                    partial.flags.get_total_allocated() + 1,
                );

                unsafe { NonNull::from_mut(&mut preallocated.allocated) }
            }
            Err(full_or_free) => {
                todo!()
            }
        }
    }

    /// Deallocate an object  from this slab.
    ///
    /// # Parameters
    ///
    /// * `idx` - The index of the object to deallocate.
    pub unsafe fn dealloc(
        &mut self,
        idx: NonMaxU16,
        cache: &mut SlabCache<T>,
    ) {
    }
}

impl<T, S> SlabDescriptor<T, S>
where
    T: Slab,
    S: SlabState<T, State = Option<NonNull<SlabDescriptor<T, S>>>>,
{
    pub fn attach(&mut self, other: &mut SlabDescriptor<T, S>) {
        other.next = self.next;
        other.state = Some(NonNull::from_ref(self));

        if let Some(mut next) = self.next {
            unsafe { next.as_mut() }.state =
                Some(NonNull::from_ref(other));
        }

        self.next = Some(NonNull::from_mut(other));
    }

    pub fn detach(&mut self) {
        if let Some(mut next) = self.next {
            unsafe { next.as_mut() }.state = self.state;
        }

        if let Some(mut prev) = self.state {
            unsafe { prev.as_mut() }.next = self.next;
        }
    }
}
