#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::missing_safety_doc)]
#![feature(ptr_alignment_type)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(macro_metavar_expr)]
#![feature(slice_ptr_get)]
#![feature(unsize)]

pub mod cache;
pub mod cache_allocator;
pub mod descriptor;
pub mod local_macros;
pub mod preallocated;
pub mod slab_address;
pub mod slab_objects;
pub mod traits;

use crate::{
    cache::SlabCaches,
    cache_allocator::CacheAllocator,
    traits::{Slab, SlabBlock},
};
use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use sync::mutex::SpinMutex;

use buddy::meta::{BuddyArena, BuddyBlock};

define_generics!(
    Generic8 = 8,
    Generic16 = 16,
    Generic32 = 32,
    Generic64 = 64,
    Generic96 = 96,
    Generic128 = 128,
    Generic192 = 192,
    Generic256 = 256,
    Generic512 = 512,
    Generic1024 = 1024,
    Generic2048 = 2048,
    Generic4096 = 4096,
    Generic8192 = 8192,
);

define_dma_generics!(
    DmaGeneric512 = 512,
    DmaGeneric1024 = 1024,
    DmaGeneric2048 = 2048,
    DmaGeneric4096 = 4096,
    DmaGeneric8192 = 8192,
);

define_slab_system!();

pub struct SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    slab_arena: &'static SpinMutex<SlabCaches<SLAB_COUNT>>,
    buddy_arena: &'static SpinMutex<Arena>,
    // Wrap in a mutex to automatically implement Sync and Send.
    _block: PhantomData<SpinMutex<Block>>,
}

pub(crate) static SLAB_ARENA: SpinMutex<SlabCaches<SLAB_COUNT>> =
    SpinMutex::new(SlabCaches::new());

impl<Block, Arena> SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    pub const fn new(buddy_arena: &'static SpinMutex<Arena>) -> Self {
        Self {
            slab_arena: &SLAB_ARENA,
            buddy_arena,
            _block: PhantomData,
        }
    }

    /// The allocator for `T`s, from `T`'s slab cache.
    pub const fn cache<T: Slab>(
        &self,
    ) -> CacheAllocator<'_, T, Block, Arena> {
        CacheAllocator::new(self)
    }

    /// Allocates one `T`.
    pub fn kmalloc<T: Slab>(&self) -> Result<NonNull<T>, AllocError> {
        self.cache::<T>()
            .allocate(Layout::new::<T>())
            .map(NonNull::cast)
    }

    /// Frees `ptr`, returning its object to its slab.
    ///
    /// # Safety
    ///
    /// `ptr` must come from `kmalloc::<T>` and not be freed since.
    pub unsafe fn kfree<T: Slab>(&self, ptr: NonNull<T>) {
        unsafe {
            self.cache::<T>().deallocate(ptr.cast(), Layout::new::<T>())
        }
    }

    /// Allocates a DMA buffer for `layout`, from the smallest DMA size
    /// that fits it.
    pub fn dma_alloc(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        match_dma_generic!(
            layout.pad_to_align().size().max(layout.align()),
            T => {
                self.cache::<T>().allocate(layout)
            },
            _ => Err(AllocError)
        )
    }

    /// Frees the DMA buffer at `ptr`, returning it to its slab.
    ///
    /// # Safety
    ///
    /// `ptr` must come from `dma_alloc(layout)` and not be freed since.
    pub unsafe fn dma_free(&self, ptr: NonNull<u8>, layout: Layout) {
        match_dma_generic!(
            layout.pad_to_align().size().max(layout.align()),
            T => {
                // SAFETY: the caller's contract carries over to the cache
                // that allocated `ptr`.
                unsafe { self.cache::<T>().deallocate(ptr, layout) }
            },
            _ => {
                // SAFETY: `layout` came from `dma_alloc`, which found a
                // size for it.
                unsafe { core::hint::unreachable_unchecked() }
            }
        )
    }
}

// SAFETY: a layout always picks the same generic size, so a block returns
// to the cache it came from.
unsafe impl<Block, Arena> Allocator for SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    /// Serves `layout` from the smallest generic size that fits it.
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        match_generic!(
            layout.pad_to_align().size().max(layout.align()),
            T => {
                self.cache::<T>().allocate(layout)
            },
            _ => Err(AllocError)
        )
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        match_generic!(
            layout.pad_to_align().size().max(layout.align()),
            T => {
                // SAFETY: the caller's contract carries over to the cache
                // that allocated `ptr`.
                unsafe { self.cache::<T>().deallocate(ptr, layout) }
            },
            _ => {
                // SAFETY: `layout` came from `allocate`, which found a size
                // for it.
                unsafe { core::hint::unreachable_unchecked() }
            }
        )
    }
}
