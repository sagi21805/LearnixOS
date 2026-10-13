extern crate alloc;

pub mod free;
pub mod meta;
pub mod partial;
pub mod state;

pub use state::*;

use crate::{
    descriptor::meta::FullFreeMeta, preallocated::PreAllocated,
    slab_address::SlabAddress, traits::Slab,
};
use core::ptr::NonNull;
use extend::ext;
use typestate_groups::{
    ByMut, CastableState, Isomorphic, MorphFrom, Morphic, WithState,
    group_impl, group_trait, typestate,
};

#[typestate(state = S, unsafe_transmute = true)]
pub struct SlabDescriptor<T: Slab, S: SlabState> {
    pub state: S::Meta,

    // TODO: Check the possibility to not save the length here because it
    // is already managed by a freelist so the len may not be needed.
    //
    // Moreover the local T holds the const order in which we allocate for
    // the array so the size can always be calculated.
    pub objects: NonNull<PreAllocated<T>>,
    pub next: Option<NonNull<SlabDescriptor<T, Erased>>>,
}

pub enum SlabStateKind {
    Partial,
    Free,
    Full,
}

impl<T: Slab, S: SlabState> SlabDescriptor<T, S> {
    /// A link to this descriptor, for another descriptor's `next`.
    pub(crate) fn as_link(&mut self) -> NonNull<SlabDescriptor<T, Erased>>
    where
        Self: CastableState<
                Erased,
                ByMut,
                Target = SlabDescriptor<T, Erased>,
            >,
    {
        NonNull::from_mut(self.cast_state_mut::<Erased>())
    }
}

#[ext(name = ErasedLink)]
pub(crate) impl<T: Slab> NonNull<SlabDescriptor<T, Erased>> {
    /// The descriptor this link points at, in state `S`.
    ///
    /// # Safety
    ///
    /// The descriptor must be live, and no other reference to it may be
    /// used while the returned one is.
    unsafe fn as_state_mut<'a, S: SlabState>(
        mut self,
    ) -> &'a mut SlabDescriptor<T, S>
    where
        SlabDescriptor<T, Erased>:
            CastableState<S, ByMut, Target = SlabDescriptor<T, S>>,
    {
        // SAFETY: the caller guarantees the descriptor is live and
        // unaliased.
        unsafe { self.as_mut() }.cast_state_mut::<S>()
    }
}

/// A descriptor leaving its list starts the detached state with that
/// state's default metadata and no link.
impl<T: Slab, S: SlabState, D> MorphFrom<SlabDescriptor<T, S>>
    for SlabDescriptor<T, D>
where
    D: SlabState + SlabRoleGroupMember<DetachedRole>,
    D::Meta: Default,
{
    fn morph_from(src: SlabDescriptor<T, S>) -> Self {
        SlabDescriptor {
            state: D::Meta::default(),
            objects: src.objects,
            next: None,
        }
    }
}

impl<T: Slab, S> SlabDescriptor<T, S>
where
    S: SlabState + SlabList + SlabRoleGroupMember<DetachedRole>,
{
    /// Makes this descriptor the head of its list, for a list that has
    /// none.
    pub(crate) fn as_head(&mut self) -> &mut SlabDescriptor<T, S::Head>
    where
        Self: CastableState<
                S::Head,
                ByMut,
                Target = SlabDescriptor<T, S::Head>,
            >,
    {
        debug_assert!(self.next.is_none());
        self.cast_state_mut::<S::Head>()
    }
}

impl<T: Slab, S> SlabDescriptor<T, S>
where
    S: SlabState<Meta = FullFreeMeta>
        + SlabList
        + SlabRoleGroupMember<AttachedRole>,
{
    /// Unlinks this descriptor from its list.
    pub(crate) fn detach(&mut self) -> &mut SlabDescriptor<T, S::Detached>
    where
        Self: CastableState<
                S::Detached,
                ByMut,
                Target = SlabDescriptor<T, S::Detached>,
            >,
        SlabDescriptor<T, S::Detached>: MorphFrom<Self>,
        SlabDescriptor<T, Erased>:
            CastableState<S, ByMut, Target = SlabDescriptor<T, S>>,
    {
        if let Some(next) = self.next {
            // SAFETY: `next` points at a live node of this list, which
            // nothing else borrows while the list is locked.
            unsafe { next.as_state_mut::<S>() }.state = self.state;
        }

        // SAFETY: `prev` is zero or the address of a live descriptor.
        if let Some(mut prev) =
            unsafe { self.state.get_prev().as_non_null() }
        {
            // SAFETY: `prev` points at a live node of this list, which
            // nothing else borrows while the list is locked.
            unsafe { prev.as_mut() }.next = self.next;
        }

        self.morph_mut::<S::Detached>()
    }
}

/// Links a detached descriptor of the same list after this head.
#[group_trait(by = SlabState)]
pub(crate) trait Attach<T: Slab>:
    WithState<State: SlabList>
{
    fn attach<'a>(
        &mut self,
        other: &'a mut SlabDescriptor<
            T,
            <Self::State as SlabList>::Detached,
        >,
    ) -> &'a mut SlabDescriptor<T, <Self::State as SlabList>::Attached>;
}

#[group_impl(LinkedStates, state = S)]
impl<T: Slab, S: SlabState + SlabList + SlabRoleGroupMember<HeadRole>>
    Attach<T> for SlabDescriptor<T, S>
where
    S::Attached: SlabState<Meta = FullFreeMeta>,
    Self: CastableState<
            Erased,
            ByMut,
            State = S,
            Target = SlabDescriptor<T, Erased>,
        >,
    SlabDescriptor<T, S::Detached>: CastableState<
            S::Attached,
            ByMut,
            Target = SlabDescriptor<T, S::Attached>,
        >,
    SlabDescriptor<T, S::Attached>:
        CastableState<Erased, ByMut, Target = SlabDescriptor<T, Erased>>,
    SlabDescriptor<T, Erased>: CastableState<
            S::Attached,
            ByMut,
            Target = SlabDescriptor<T, S::Attached>,
        >,
{
    fn attach<'a>(
        &mut self,
        other: &'a mut SlabDescriptor<T, S::Detached>,
    ) -> &'a mut SlabDescriptor<T, S::Attached> {
        let attached = other.cast_state_mut::<S::Attached>();
        attached.next = self.next;
        attached
            .state
            .set_prev(SlabAddress::from_non_null(self.as_link()));

        let link = attached.as_link();
        if let Some(next) = self.next {
            // SAFETY: `next` points at a live node of this list, which
            // nothing else borrows while the list is locked.
            unsafe { next.as_state_mut::<S::Attached>() }
                .state
                .set_prev(SlabAddress::from_non_null(link));
        }

        self.next = Some(link);
        attached
    }
}

#[group_impl(PartialStates, state = S)]
impl<T: Slab, S: SlabState + SlabList + SlabRoleGroupMember<HeadRole>>
    Attach<T> for SlabDescriptor<T, S>
where
    SlabDescriptor<T, S::Detached>: CastableState<
            S::Attached,
            ByMut,
            Target = SlabDescriptor<T, S::Attached>,
        >,
    SlabDescriptor<T, S::Attached>:
        CastableState<Erased, ByMut, Target = SlabDescriptor<T, Erased>>,
{
    fn attach<'a>(
        &mut self,
        other: &'a mut SlabDescriptor<T, S::Detached>,
    ) -> &'a mut SlabDescriptor<T, S::Attached> {
        let attached = other.cast_state_mut::<S::Attached>();
        attached.next = self.next;
        self.next = Some(attached.as_link());
        attached
    }
}

impl<T: Slab> SlabDescriptor<T, Used> {
    pub fn is_partial(
        &self,
    ) -> Result<&SlabDescriptor<T, Partial>, &SlabDescriptor<T, Full>>
    {
        if self.state.is_partial() {
            todo!("");
            Ok(self.cast_state_ref::<Partial>())
        } else {
            todo!("");
            Err(self.cast_state_ref::<Full>())
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
            Ok(self.cast_state_mut::<Partial>())
        } else {
            todo!("");
            Err(self.cast_state_mut::<Full>())
        }
    }
}
