extern crate alloc;

pub mod free;
pub mod full;
pub mod partial;

use crate::{
    preallocated::PreAllocated,
    traits::{DetachedSlabState, Slab, SlabState},
};
use common::address_types::{Address, VirtualAddress};
use core::{num::NonZeroU64, ptr::NonNull};
use macros::bitfields;

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
    pub next: Option<NonNull<S::Next>>,
}

/// Partial slab is a slab that has some allocated objects, and some free
/// objects.
pub struct Partial;
pub struct PartialDetached;

impl<T: Slab> DetachedSlabState<T> for PartialDetached {
    type Attached = Partial;
}

impl<T: Slab> SlabState<T> for Partial {
    type Meta = PartialMeta;
    type DetachedState = PartialDetached;
}

impl<T: Slab> SlabState<T> for PartialDetached {
    type Meta = RawMeta;
    type Next = ();
    type Detached = ();
    type DetachedState = ();
}

/// Free slab is a slab that does not allocate any objects, and is
/// initialized that the first allocatable index is 0.
pub struct Free;
pub struct FreeDetached;

impl<T: Slab> DetachedSlabState<T> for FreeDetached {
    type Attached = Free;
}

impl<T: Slab> SlabState<T> for Free {
    type Meta = FullFreeMeta;
    type DetachedState = FreeDetached;
}

impl<T: Slab> SlabState<T> for FreeDetached {
    type Meta = RawMeta;
    type Next = ();
    type Detached = ();
    type DetachedState = ();
}

/// Full slab is a slab that is fully allocated.
pub struct Full;
pub struct FullDetached;

impl<T: Slab> DetachedSlabState<T> for FullDetached {
    type Attached = Full;
}

impl<T: Slab> SlabState<T> for Full {
    type Meta = FullFreeMeta;
    type DetachedState = FullDetached;
}

impl<T: Slab> SlabState<T> for FullDetached {
    type Meta = RawMeta;
    type Next = ();
    type Detached = ();
    type DetachedState = ();
}

/// A used slab may be full or partial.
///
/// This state is ment to be when trying to free an object and trying to
/// figure out the state of the slab.
pub struct Used;
impl<T: Slab> SlabState<T> for Used {
    type Meta = RawMeta;
    type Next = ();
    type Detached = ();
    type DetachedState = ();
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

const impl From<u64> for SlabAddress {
    fn from(value: u64) -> Self { Self(NonZeroU64::new(value)) }
}

const impl From<SlabAddress> for u64 {
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

/// Shared linked-list attach/detach logic for the two state kinds
/// (`Free` and `Full`) that use `FullFreeMeta` as their metadata.
impl<T, S> SlabDescriptor<T, S>
where
    T: Slab,
    S: SlabState<T, Meta = FullFreeMeta, Next = Self>,
{
    pub(crate) fn attach_linked(
        &mut self,
        other: &mut SlabDescriptor<T, S>,
    ) {
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

    pub(crate) fn detach_linked(&mut self) -> &mut S::Detached {
        if let Some(mut next) = self.next {
            unsafe { next.as_mut().state = self.state };
        }

        if let Some(mut prev) =
            unsafe { self.state.get_prev().as_non_null::<T, S>() }
        {
            unsafe { prev.as_mut() }.next = self.next;
        }

        self.next = None;

        unsafe { core::mem::transmute(self) }
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
