extern crate alloc;

use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use alloc::boxed::Box;
use buddy::meta::{BuddyArena, BuddyBlock};
use extend::ext;

use crate::{
    SlabAllocator, SlabArena,
    traits::{Slab, SlabBlock},
};

/// The one slab allocator of a program, reachable with no reference.
///
/// The program implements it on a marker type of its own, as the slab
/// crate can't name the program's static.
///
/// ```ignore
/// struct KernelSlab;
///
/// impl GlobalSlab for KernelSlab {
///     type Block = Page;
///     type Arena = PageMap;
///
///     fn slab() -> &'static SlabAllocator<Page, PageMap> { &SLAB }
/// }
/// ```
pub trait GlobalSlab: 'static {
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
