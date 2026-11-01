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
#![feature(pointer_is_aligned_to)]
#![feature(pattern_types, pattern_type_macro)]
#![expect(
    internal_features,
    reason = "`SlotIndex` is a pattern type, which is an internal feature"
)]

mod arena;
pub mod cache;
pub mod cache_allocator;
pub mod descriptor;
pub mod local_macros;
pub mod preallocated;
pub mod slab_address;
pub mod slab_objects;
pub mod slot_index;
pub mod traits;

pub use arena::SlabArena;

use crate::{
    cache::SlabCaches,
    cache_allocator::CacheAllocator,
    descriptor::{FreeDetached, SlabDescriptor},
    traits::{Slab, SlabBlock},
};
use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use sync::mutex::SpinMutex;

use buddy::{
    BuddyAllocator, BuddyAllocatorInner,
    meta::{BuddyArena, BuddyBlock},
};

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
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
{
    caches: SpinMutex<SlabCaches<SLAB_COUNT>>,
    buddy: &'static BuddyAllocator<Arena, Block>,
}

impl<Block, Arena> SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
{
    /// A slab allocator with empty caches, which takes its pages from
    /// `buddy`.
    ///
    /// No descriptor points back into the caches, so unlike the buddy
    /// allocator it needs no pinning.
    pub const fn new(
        buddy: &'static BuddyAllocator<Arena, Block>,
    ) -> Self {
        Self {
            caches: SpinMutex::new(SlabCaches::new()),
            buddy,
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

    /// A new free slab of `T`s on `T::PAGES_PER_SLAB` fresh pages from the
    /// locked buddy allocator, whose descriptor lives in the block of its
    /// first page.
    pub(crate) fn new_slab<T: Slab>(
        buddy: &mut BuddyAllocatorInner<Arena, Block>,
    ) -> Result<&mut SlabDescriptor<T, FreeDetached>, AllocError> {
        let pages = buddy.allocate(T::SLAB_LAYOUT)?.cast::<u8>();
        let mut block = buddy
            .block_of(pages)
            .expect("the buddy allocator's pages are in its arena");

        // SAFETY: the buddy allocator gave `block` out for this slab, so
        // it holds no slab in use.
        Ok(unsafe {
            block
                .as_mut()
                .unused_slab_descriptor_mut()
                .assign(SlabDescriptor::new())
        })
    }
}

// SAFETY: a layout always picks the same generic size, so a block returns
// to the cache it came from.
unsafe impl<Block, Arena> Allocator for SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
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
