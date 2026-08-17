extern crate alloc;

pub mod free;
pub mod full;
pub mod meta;
pub mod partial;

use crate::{
    descriptor::meta::{FullFreeMeta, PartialMeta, RawMeta},
    new_state,
    preallocated::PreAllocated,
    slab_address::SlabAddress,
    traits::{Slab, SlabState},
};
use core::ptr::NonNull;

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

// Partial slab is a slab that has some allocated objects, and some free
// objects.
new_state!(Partial => PartialDetached, PartialMeta);

// Free slab is a slab that does not allocate any objects, and is
// initialized that the first allocatable index is 0.
new_state!(Free => FreeDetached, FullFreeMeta);

// Full slab is a slab that is fully allocated.
new_state!(Full => FullDetached, FullFreeMeta);

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

/// Shared linked-list attach/detach logic for the two state kinds
/// (`Free` and `Full`) that use `FullFreeMeta` as their metadata.
impl<T, S> SlabDescriptor<T, S>
where
    T: Slab,
    S: SlabState<T, Meta = FullFreeMeta, Next = Self>,
    S::DetachedState: SlabState<T, Meta = FullFreeMeta>,
{
    pub(crate) fn attach_linked(
        &mut self,
        other: &mut SlabDescriptor<T, S::DetachedState>,
    ) -> &mut SlabDescriptor<T, S> {
        other.next = self.next.map(|p| p.cast());

        other
            .state
            .set_prev(SlabAddress::from_non_null(NonNull::from_ref(self)));

        if let Some(mut next) = self.next {
            unsafe { next.as_mut() }.state.set_prev(
                SlabAddress::from_non_null(NonNull::from_mut(other)),
            );
        }

        let mut attached =
            NonNull::from_mut(unsafe { core::mem::transmute(other) });

        self.next = Some(attached);

        unsafe { attached.as_mut() }
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
            todo!("");
            Ok(unsafe { core::mem::transmute(self) })
        } else {
            todo!("");
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
            todo!("");
            Ok(unsafe { core::mem::transmute(self) })
        } else {
            todo!("");
            Err(unsafe { core::mem::transmute(self) })
        }
    }
}
