extern crate alloc;

use crate::{preallocated::PreAllocated, traits::Slab};
use alloc::alloc::{Layout, alloc};
use common::{
    address_types::{Address, VirtualAddress},
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
};
use core::{mem::size_of, num::NonZeroU64, ptr::NonNull};
use macros::bitfields;
use nonmax::NonMaxU16;
use x86::structures::mbr::PartitionTableEntry;

#[repr(C)]
pub struct SlabDescriptor<T, S>
where
    T: Slab,
    S: SlabState<T>,
{
    pub state: S::Meta,

    // TODO: Check the possibility to not save the length here because it
    // is already managed by a freelist so the len may not be needed.
    //
    // Moreover the local T holds the const order in which we allocate for
    // the array so the size can always be calculated.
    pub objects: NonNull<PreAllocated<T>>,
    pub next: Option<NonNull<SlabDescriptor<T, S>>>,
}

// TODO: Seal trait
pub trait SlabState<T: Slab> {
    type Meta: Sized;
}

/// Partial slab is a slab that has some allocated objects, and some free
/// objects.
pub struct Partial;
impl<T: Slab> SlabState<T> for Partial {
    type Meta = PartialMeta;
}

/// Free slab is a slab that does not allocate any objects, and is
/// initialized that the first allocatable index is 0.
pub struct Free;
impl<T: Slab> SlabState<T> for Free {
    type Meta = FullFreeMeta;
}

/// Full slab is a slab that is fully allocated.
pub struct Full;
impl<T: Slab> SlabState<T> for Full {
    type Meta = FullFreeMeta;
}

/// A used slab may be full or partial.
///
/// This state is ment to be when trying to free an object and trying to
/// figure out the state of the slab.
pub struct Used;
impl<T: Slab> SlabState<T> for Used {
    type Meta = RawMeta;
}

pub trait Attach {
    fn attach(&mut self, other: &mut Self);
}

pub trait Detach {
    fn detach(&mut self);
}

pub enum SlabStateKind {
    Partial,
    Free,
    Full,
}

#[bitfields]
pub struct PartialMeta {
    pub next_free_idx: B16,
    pub total_allocated: B31,
    pub partial: B1,
}

#[bitfields]
pub struct RawMeta {
    #[flag(r)]
    pub reserved: B63,
    pub partial: B1,
}

#[derive(Debug, Clone, Copy)]
pub struct SlabAddress(Option<NonZeroU64>);

#[rustfmt::skip]
impl const From<u64> for SlabAddress {
    fn from(value: u64) -> Self {
        Self(NonZeroU64::new(value))
    }
}

#[rustfmt::skip]
impl const From<SlabAddress> for u64 {
    fn from(value: SlabAddress) -> Self {
        match value.0 {
            Some(v) => v.get(),
            None => 0,
        }
    }
}

impl SlabAddress {
    pub unsafe fn as_non_null<T: Slab, S: SlabState<T>>(
        &self,
    ) -> Option<NonNull<SlabDescriptor<T, S>>> {
        unsafe {
            Some(
                VirtualAddress::new_unchecked(self.0?.get() as usize)
                    .as_non_null(),
            )
        }
    }

    pub fn from_non_null<T: Slab, S: SlabState<T>>(
        ptr: NonNull<SlabDescriptor<T, S>>,
    ) -> Self {
        Self(NonZeroU64::try_from(ptr.addr()).ok())
    }
}

#[bitfields]
pub struct FullFreeMeta {
    #[flag(flag_type = SlabAddress)]
    pub prev: B48,
    #[flag(r)]
    pub reserved: B15,
    pub partial: B1,
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
                state: FullFreeMeta::new()
                    .prev(SlabAddress(None))
                    .partial(false),
                objects: objects.cast(),
                next,
            }
        }
    }
}

impl<T: Slab> SlabDescriptor<T, Partial> {
    /// Allocate an object from this slab returning the allocated object
    /// and state of the slab.
    ///
    /// # Parameters
    ///
    /// * `head` - The head of the full slab descriptor. This slab will be
    ///   attached to it, if the allocation will make this slab full.
    pub fn alloc(&mut self) -> (NonNull<T>, SlabStateKind) {
        debug_assert!(
            self.state.is_partial(),
            "Compiletime state does not match runtime state"
        );

        let idx = self.state.get_next_free_idx() as usize;
        let preallocated =
            unsafe { self.objects.add(idx as usize).as_mut() };

        let mut final_state = SlabStateKind::Partial;

        self.state
            .set_total_allocated(self.state.get_total_allocated() + 1);

        match unsafe { preallocated.next_free_idx } {
            Some(idx) => {
                self.state.set_next_free_idx(idx.get());
            }
            None => {
                final_state = SlabStateKind::Full;
            }
        }

        unsafe {
            (NonNull::from_mut(&mut preallocated.allocated), final_state)
        }
    }

    /// Deallocate an object  from this slab.
    ///
    /// # Parameters
    ///
    /// * `idx` - The index of the object to deallocate.
    pub unsafe fn dealloc(&mut self, idx: NonMaxU16) -> SlabStateKind {
        unsafe {
            self.objects.add(idx.get() as usize).as_mut().next_free_idx =
                NonMaxU16::new(self.state.get_next_free_idx());
        }

        self.state.set_next_free_idx(idx.get());

        self.state
            .set_total_allocated(self.state.get_total_allocated() - 1);

        if self.state.get_total_allocated() == 0 {
            SlabStateKind::Free
        } else {
            SlabStateKind::Partial
        }
    }
}

impl<T> SlabDescriptor<T, Free>
where
    T: Slab,
{
    pub fn alloc(&mut self) -> (NonNull<T>, SlabStateKind) {
        self.detach();

        let partial: &mut SlabDescriptor<T, Partial> =
            unsafe { core::mem::transmute(self) };

        partial.state = PartialMeta::new()
            .partial(true)
            .next_free_idx(0)
            .total_allocated(0);

        partial.alloc()
    }
}

impl<T, S> Attach for SlabDescriptor<T, S>
where
    T: Slab,
    S: SlabState<T, Meta = FullFreeMeta>,
{
    fn attach(&mut self, other: &mut SlabDescriptor<T, S>) {
        other.next = self.next;
        other
            .state
            .set_prev(SlabAddress::from_non_null(NonNull::from_ref(self)));

        if let Some(mut next) = self.next {
            unsafe { next.as_mut() }.state.set_prev(
                SlabAddress::from_non_null(NonNull::from_mut(other)),
            );
        }

        self.next = Some(NonNull::from_mut(other));
    }

}

impl<T, S> Detach for SlabDescriptor<T, S>
where
    T: Slab,
    S: SlabState<T, Meta = FullFreeMeta>,
{
    fn detach(&mut self) {
        if let Some(mut next) = self.next {
            unsafe { next.as_mut().state = self.state };
        }

        if let Some(mut prev) =
            unsafe { self.state.get_prev().as_non_null() }
        {
            unsafe { prev.as_mut() }.next = self.next;
        }

        self.next = None;
    }
}

impl<T: Slab> Attach for SlabDescriptor<T, Partial> {
    fn attach(&mut self, other: &mut SlabDescriptor<T, Partial>) {
        other.next = self.next;

        self.next = Some(NonNull::from_mut(other));
    }
}



impl<T: Slab> SlabDescriptor<T, Used> {
    pub fn is_partial(
        &self,
    ) -> Result<&SlabDescriptor<T, Partial>, &SlabDescriptor<T, Full>>
    {
        if self.state.is_partial() {
            Ok(unsafe { core::mem::transmute(self) })
        } else {
            Err(unsafe { core::mem::transmute(self) })
        }
    }

    pub fn is_partial_mut(
        &mut self,
    ) -> Result<
        &mut SlabDescriptor<T, Partial>,
        &mut SlabDescriptor<T, Full>,
    > {
        if self.state.is_partial() {
            Ok(unsafe { core::mem::transmute(self) })
        } else {
            Err(unsafe { core::mem::transmute(self) })
        }
    }
}
