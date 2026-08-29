use core::ptr::NonNull;

use common::{
    address_types::VirtualAddress, constants::REGULAR_PAGE_SIZE,
    enums::PageSize,
};
use x86::structures::paging::PageEntryFlags;

use crate::descriptor::{
    FreeDetached, FullDetached, PartialDetached, SlabDescriptor, Used,
    meta::RawMeta,
};

/// Get the position on the slab array, for a slab of the given type.
///
/// Shouldn't implement this trait manually; it is implemented
/// via the `define_slab_system` macro.
pub trait Slab: SlabPosition + SlabFlags + 'static {}

impl Slab for () {}

pub trait SlabPosition: Sized {
    const SLAB_POSITION: usize;
    const PAGES_PER_SLAB: usize = size_of::<Self>()
        .next_multiple_of(REGULAR_PAGE_SIZE)
        / REGULAR_PAGE_SIZE;
    const OBJECT_PER_SLAB: usize =
        Self::PAGES_PER_SLAB / size_of::<Self>();
}

impl SlabPosition for () {
    const SLAB_POSITION: usize = usize::MAX;
}

pub trait SlabFlags: SlabPosition {
    const PFLAGS: PageEntryFlags;
    const PSIZE: PageSize;
}

impl<T: SlabPosition> SlabFlags for T {
    default const PFLAGS: PageEntryFlags =
        PageEntryFlags::regular_page_flags();

    default const PSIZE: PageSize = PageSize::Regular;
}

pub(crate) unsafe trait SlabState<T: Slab>: Sized {
    /// The type the `self.next` pointer will point to.
    ///
    /// This is useful because it allows to set invalid next pointer to
    /// detached slabs.
    type Next: Sized = SlabDescriptor<T, Self>;

    /// Type that holds the metadata of the slab.
    ///
    /// This type should be the same size of u64.
    type Meta: Sized;

    const _ASSERT_META_SIZE: () = assert!(
        core::mem::size_of::<Self::Meta>() == core::mem::size_of::<u64>()
    );
}

unsafe impl<T: Slab> SlabState<T> for () {
    type Meta = RawMeta;
    type Next = ();
}

// TODO: Seal trait
pub(crate) unsafe trait AttachedSlabState<T: Slab>:
    SlabState<T>
{
    /// The detached state of this state. This type should be zero sized.
    type DetachedState: DetachedSlabState<T>;

    type HeadState: HeadSlabState<T>;

    const _ASSERT_DETACHED_SIZE: () =
        assert!(core::mem::size_of::<Self::DetachedState>() == 0);
}

/// A slabdescriptor that is detached from the slab cache.
pub(crate) unsafe trait DetachedSlabState<T: Slab>:
    SlabState<T>
{
    /// The attached state of this detached state.
    type AttachedState: AttachedSlabState<T>;

    type HeadState: HeadSlabState<T>;
}

pub(crate) unsafe trait HeadSlabState<T: Slab>:
    AttachedSlabState<T>
{
    type AttachedState: AttachedSlabState<T>;

    type Attached = SlabDescriptor<T, Self::AttachedState>;

    type DetachedState: DetachedSlabState<T>;

    type Detached =
        SlabDescriptor<T, <Self as HeadSlabState<T>>::DetachedState>;
}

unsafe impl<T: Slab> HeadSlabState<T> for () {
    type AttachedState = ();

    type DetachedState = ();
}

unsafe impl<T: Slab> DetachedSlabState<T> for () {
    type AttachedState = ();
    type HeadState = ();
}

pub(crate) unsafe trait AttachedSlab<T: Slab, S: AttachedSlabState<T>> {
    type Head: HeadSlab<T, S::HeadState> = SlabDescriptor<T, S::HeadState>;

    type Detached: DetachedSlab<T, S::DetachedState> =
        SlabDescriptor<T, S::DetachedState>;
}

unsafe impl<T: Slab> AttachedSlabState<T> for () {
    type DetachedState = ();
    type HeadState = ();
}

pub trait Generic {
    const START: usize;
    const END: usize;

    fn size(&self) -> usize;
}

pub trait DmaGeneric {
    const START: usize;
    const END: usize;

    fn size(&self) -> usize;
}

pub trait SlabBlock {
    fn from_address(address: VirtualAddress) -> Self;

    /// Retrive a reference to the [`SlabDescriptor`] from the block.
    ///
    /// # Safety
    ///
    /// The `T` that this function gets will cast the [`SlabDescriptor`]
    /// that in this block unconditionality which in unsafe.
    fn slab_descriptor<T: Slab>(&self) -> &SlabDescriptor<T, Used>;

    /// Retrive a mutable reference [`SlabDescriptor`] from the block.
    ///
    /// # Safety
    ///
    /// The `T` that this function gets will cast the [`SlabDescriptor`]
    /// that in this block unconditionality which in unsafe.
    fn slab_descriptor_mut<T: Slab>(
        &mut self,
    ) -> &mut SlabDescriptor<T, Used>;
}

pub(crate) trait Attach<T: Slab, S: AttachedSlabState<T>>:
    AttachedSlab<T, S>
{
    /// Attach a slab in the free state to this slab.
    fn attach_free(
        &mut self,
        other: &mut SlabDescriptor<T, FreeDetached>,
    ) -> &mut SlabDescriptor<T, S>;

    /// Attach a slab in the full state to this slab.
    fn attach_full(
        &mut self,
        other: &mut SlabDescriptor<T, FullDetached>,
    ) -> &mut SlabDescriptor<T, S>;

    /// Attach a slab in the partial state into this slab.
    fn attach_partial(
        &mut self,
        other: &mut SlabDescriptor<T, PartialDetached>,
    ) -> &mut SlabDescriptor<T, S>;
}

pub(crate) trait Detach<T: Slab, S: AttachedSlabState<T>>:
    AttachedSlab<T, S>
{
    fn detach(&mut self) -> &mut Self::Detached;
}

/// A slab descriptor that is a detached state.
pub(crate) unsafe trait DetachedSlab<T: Slab, S: DetachedSlabState<T>> {
    type Attached: AttachedSlab<T, S::AttachedState> =
        SlabDescriptor<T, S::AttachedState>;
    type Head: HeadSlab<T, S::HeadState> = SlabDescriptor<T, S::HeadState>;
}

unsafe impl<T: Slab, S: DetachedSlabState<T>> DetachedSlab<T, S>
    for SlabDescriptor<T, S>
{
}

pub(crate) unsafe trait HeadSlab<T: Slab, S: HeadSlabState<T>> {
    type Attached: AttachedSlab<T, S::AttachedState> =
        SlabDescriptor<T, S::AttachedState>;
}

unsafe impl<T: Slab, S: HeadSlabState<T>> HeadSlab<T, S>
    for SlabDescriptor<T, S>
{
}

unsafe impl<T: Slab, S: AttachedSlabState<T>> AttachedSlab<T, S>
    for SlabDescriptor<T, S>
{
    type Detached = SlabDescriptor<T, S::DetachedState>;
    type Head = SlabDescriptor<T, S::HeadState>;
}

/// Convert from one detached state into another.
pub(crate) unsafe trait ConvertInplace<T, S, D>:
    DetachedSlab<T, S>
where
    T: Slab,
    S: DetachedSlabState<T>,
    D: DetachedSlabState<T>,
{
    fn convert_to(
        &mut self,
        meta: <D as SlabState<T>>::Meta,
    ) -> &mut SlabDescriptor<T, D>;
}

unsafe impl<T, S, D> ConvertInplace<T, S, D> for SlabDescriptor<T, S>
where
    T: Slab,
    S: DetachedSlabState<T, Meta = D::Meta>,
    D: DetachedSlabState<T>,
{
    fn convert_to(
        &mut self,
        meta: <D as SlabState<T>>::Meta,
    ) -> &mut SlabDescriptor<T, D> {
        // TODO: maybe add function to ensure runtime state with compile
        // time state.
        self.state = meta;
        unsafe { core::mem::transmute(self) }
    }
}

unsafe impl<T, S, D> ConvertInplace<T, S, D> for SlabDescriptor<T, S>
where
    T: Slab,
    S: DetachedSlabState<T>,
    D: DetachedSlabState<T>,
{
    fn convert_to(
        &mut self,
        meta: <D as SlabState<T>>::Meta,
    ) -> &mut SlabDescriptor<T, D> {
        let transmuted = unsafe {
            core::mem::transmute::<
                &mut SlabDescriptor<T, S>,
                &mut SlabDescriptor<T, D>,
            >(self)
        };
        transmuted.state = meta;
        transmuted
    }
}

/// Make a detached node the head of a list of his head type.
///
/// In the case that the initial node on the cache list does not exist, it
/// needs to `attach himself` by making himself the head of the list.
pub(crate) unsafe trait IntoHead<T: Slab, S: DetachedSlabState<T>>:
    DetachedSlab<T, S>
{
    fn into_head(&mut self) -> &mut SlabDescriptor<T, S::HeadState> {
        unsafe { NonNull::from_mut(self).cast().as_mut() }
    }
}

unsafe impl<T: Slab, S: DetachedSlabState<T>, U: DetachedSlab<T, S>>
    IntoHead<T, S> for U
{
}
