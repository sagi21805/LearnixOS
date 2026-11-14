extern crate alloc;

use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use alloc::{
    boxed::Box, collections::VecDeque, rc::Rc, sync::Arc, vec::Vec,
};
use buddy::meta::{BuddyArena, BuddyBlock};
use extend::ext;

use crate::{
    SlabAllocator, SlabArena,
    traits::{Slab, SlabBlock},
};

/// The one slab allocator of a program, reachable with no reference.
///
/// # Safety
///
/// Because this allocator is global, it should be implemented on a static
/// and that it would be the only allocator.
pub unsafe trait GlobalSlab: 'static {
    type Block: BuddyBlock + SlabBlock + 'static;
    type Arena: BuddyArena<Self::Block> + SlabArena + 'static;

    fn slab() -> &'static SlabAllocator<Self::Block, Self::Arena>;
}

/// Allocates `T`s from their slab cache in the slab of `G`.
///
/// It is zero-sized, so a box that uses it is one pointer wide.
pub struct StaticCacheAllocator<T: Slab, G: GlobalSlab>(
    PhantomData<fn() -> (T, G)>,
);

impl<T: Slab, G: GlobalSlab> StaticCacheAllocator<T, G> {
    pub const fn new() -> Self { Self(PhantomData) }
}

impl<T: Slab, G: GlobalSlab> Default for StaticCacheAllocator<T, G> {
    fn default() -> Self { Self::new() }
}

impl<T: Slab, G: GlobalSlab> Clone for StaticCacheAllocator<T, G> {
    fn clone(&self) -> Self { *self }
}

impl<T: Slab, G: GlobalSlab> Copy for StaticCacheAllocator<T, G> {}

// SAFETY: every handle reaches the same cache of the same allocator.
unsafe impl<T: Slab, G: GlobalSlab> Allocator
    for StaticCacheAllocator<T, G>
{
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        G::slab().cache::<T>().allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: the caller's contract carries over to the cache.
        unsafe { G::slab().cache::<T>().deallocate(ptr, layout) }
    }
}

/// A box of a `T` from its own slab cache in the slab of `G`, one pointer
/// wide.
///
/// ```ignore
/// let inode = SlabBox::<Inode, KernelSlab>::try_new_slab(Inode::new())?;
/// drop(inode); // back to the cache of `Inode`
/// ```
pub type SlabBox<T, G> = Box<T, StaticCacheAllocator<T, G>>;

#[ext(name = SlabBoxNew)]
pub impl<T: Slab, G: GlobalSlab> SlabBox<T, G> {
    /// Moves `value` into a slot of `T`'s cache.
    fn try_new_slab(value: T) -> Result<SlabBox<T, G>, AllocError> {
        Box::try_new_in(value, StaticCacheAllocator::new())
    }
}

/// Allocates by layout from the generic sizes of the slab of `G`, as
/// [`SlabAllocator`](crate::SlabAllocator) does.
///
/// A collection allocates layouts other than its element's, so it can't
/// use the element's own cache. A `Vec<T>` asks for arrays of `T`, and an
/// `Arc<T>` for a `T` behind its two counters.
///
/// It is zero-sized, so a collection that uses it is as wide as one on
/// the global allocator.
pub struct StaticSlabAllocator<G: GlobalSlab>(PhantomData<fn() -> G>);

impl<G: GlobalSlab> StaticSlabAllocator<G> {
    pub const fn new() -> Self { Self(PhantomData) }
}

impl<G: GlobalSlab> Default for StaticSlabAllocator<G> {
    fn default() -> Self { Self::new() }
}

impl<G: GlobalSlab> Clone for StaticSlabAllocator<G> {
    fn clone(&self) -> Self { *self }
}

impl<G: GlobalSlab> Copy for StaticSlabAllocator<G> {}

// SAFETY: every handle forwards to the one allocator of `G`.
unsafe impl<G: GlobalSlab> Allocator for StaticSlabAllocator<G> {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        G::slab().allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: the caller's contract carries over to the slab.
        unsafe { G::slab().deallocate(ptr, layout) }
    }
}

/// A vector whose buffer comes from the generic sizes of the slab of `G`.
///
/// ```ignore
/// let mut ids = SlabVec::<u32, KernelSlab>::new_in(Default::default());
/// ids.push(7);
/// ```
pub type SlabVec<T, G> = Vec<T, StaticSlabAllocator<G>>;

/// A ring buffer whose buffer comes from the generic sizes of the slab of
/// `G`.
pub type SlabVecDeque<T, G> = VecDeque<T, StaticSlabAllocator<G>>;

/// An atomically counted `T` from the generic sizes of the slab of `G`.
pub type SlabArc<T, G> = Arc<T, StaticSlabAllocator<G>>;

/// A counted `T` from the generic sizes of the slab of `G`.
pub type SlabRc<T, G> = Rc<T, StaticSlabAllocator<G>>;
