use common::{
    address_types::VirtualAddress, constants::REGULAR_PAGE_SIZE,
    enums::PageSize,
};
use x86::structures::paging::PageEntryFlags;

use crate::descriptor::{
    Free, FreeDetached, Full, FullDetached, Partial, PartialDetached,
    RawMeta, SlabDescriptor, Used,
};

/// Get the position on the slab array, for a slab of the given type.
///
/// Shouldn't implement this trait manually; it is implemented
/// via the `define_slab_system` macro.
pub trait Slab: SlabPosition + SlabFlags {}

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

// TODO: Seal trait
pub trait SlabState<T: Slab>: Sized {
    /// Type that holds the metadata of the slab.
    ///
    /// This type should be the same size of u64.
    type Meta: Sized;

    const ASSERT_META_SIZE: () = assert!(
        core::mem::size_of::<Self::Meta>() == core::mem::size_of::<u64>()
    );

    /// The type the `self.next` pointer will point to.
    ///
    /// This is useful because it allows to set invalid next pointer to
    /// detached slabs.
    type Next: Sized = SlabDescriptor<T, Self>;

    /// The detached state of this state. This type should be zero sized.
    type DetachedState: DetachedSlabState<T>;

    type Detached = SlabDescriptor<T, Self::DetachedState>;

    const ASSERT_DETACHED_SIZE: () =
        assert!(core::mem::size_of::<Self::Detached>() == 0);
}

/// A slabdescriptor that is detached from the slab cache.
pub trait DetachedSlabState<T: Slab>: SlabState<T, Next = ()> {
    /// The attached state of this detached state.
    type Attached: SlabState<T>;
}

impl<T: Slab> DetachedSlabState<T> for () {
    type Attached = ();
}

pub(crate) unsafe trait AttachedSlab<T: Slab, S: SlabState<T>> {}

impl<T: Slab> SlabState<T> for () {
    type Meta = RawMeta;
    type Next = ();
    type Detached = ();
    type DetachedState = ();
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

pub trait Attach<T: Slab, S: SlabState<T>>: AttachedSlab<T, S> {
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

pub trait Detach<T: Slab, S: SlabState<T>> {
    fn detach(&mut self) -> &mut S::Detached;
}

/// A slab descriptor that is a detached state.
pub trait DetachedSlab<T: Slab, S: DetachedSlabState<T>> {}

pub trait ConvertInplace<T, D, S>: DetachedSlab<T, S>
where
    T: Slab,
    D: DetachedSlabState<T>,
    S: DetachedSlabState<T>,
{
    fn convert_inplace(
        &mut self,
        meta: <D::Attached as SlabState<T>>::Meta,
    ) -> &mut SlabDescriptor<T, D>;
}

/// Make a detached node an attached one.
///
/// In the case that the initial node on the cache list does not exist, it
/// needs to `attach himself`
pub(crate) unsafe trait SelfAttach<T: Slab, S: DetachedSlabState<T>>:
    DetachedSlab<T, S>
{
    fn attach_self(&mut self) -> &mut SlabDescriptor<T, S::Attached>;
}
