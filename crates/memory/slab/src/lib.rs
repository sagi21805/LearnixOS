#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(allocator_ext)]
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
    reason = "`ObjectIndex` is a pattern type, which is an internal \
              feature"
)]

mod arena;
pub mod cache;
pub mod cache_allocator;
pub mod descriptor;
pub mod local_macros;
pub mod object_index;
pub mod preallocated;
pub mod slab_objects;
pub mod static_allocator;
pub mod traits;

pub use arena::{SlabArena, SlabFlags};
pub use static_allocator::{
    GlobalSlab, SlabArc, SlabBox, SlabBoxNew, SlabRc, SlabVec,
    SlabVecDeque, StaticCacheAllocator, StaticSlabAllocator,
    StaticSlabDmaAllocator,
};

use crate::{
    cache::{CachedIn, ExternalCaches, SlabCaches},
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
    BuddyAllocator,
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

/// The types one slab allocator works with, which always travel
/// together.
pub trait SlabSystem: Sized + 'static {
    /// The buddy block, which holds the descriptor of a slab.
    type Block: BuddyBlock + SlabBlock + 'static;
    /// The buddy arena the pages of the slabs come from.
    type Arena: BuddyArena<Self::Block> + SlabArena + 'static;
    /// The caches of the program's own slab types.
    type Caches: ExternalCaches;
}

pub struct SlabAllocator<S: SlabSystem> {
    caches: SpinMutex<SlabCaches<S::Caches>>,
    buddy: &'static BuddyAllocator<S::Arena, S::Block>,
}

impl<S: SlabSystem> SlabAllocator<S> {
    /// A slab allocator with empty caches, which takes its pages from
    /// `buddy`.
    ///
    /// No descriptor points back into the caches, so unlike the buddy
    /// allocator it needs no pinning.
    pub const fn new(
        buddy: &'static BuddyAllocator<S::Arena, S::Block>,
    ) -> Self {
        Self {
            caches: SpinMutex::new(SlabCaches::new()),
            buddy,
        }
    }

    /// The allocator for `T`s, from `T`'s slab cache.
    pub const fn cache<T: CachedIn<S::Caches>>(
        &self,
    ) -> CacheAllocator<'_, T, S> {
        CacheAllocator::new(self)
    }

    /// Allocates one `T`.
    pub fn kmalloc<T: CachedIn<S::Caches>>(
        &self,
    ) -> Result<NonNull<T>, AllocError> {
        self.cache::<T>()
            .allocate(Layout::new::<T>())
            .map(NonNull::cast)
    }

    /// Frees `ptr`, returning its object to its slab.
    ///
    /// # Safety
    ///
    /// `ptr` must come from `kmalloc::<T>` and not be freed since.
    pub unsafe fn kfree<T: CachedIn<S::Caches>>(&self, ptr: NonNull<T>) {
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

    /// A new free slab of `T`s on `T::PAGES_PER_SLAB` fresh pages from
    /// buddy, whose descriptor lives in the block of its first page.
    #[expect(
        clippy::mut_from_ref,
        reason = "the descriptor is in pages buddy just handed out, \
                  which nothing else reaches"
    )]
    pub(crate) fn new_slab<T: Slab>(
        &self,
    ) -> Result<&mut SlabDescriptor<T, FreeDetached>, AllocError> {
        let block = self.buddy.allocate_block(T::SLAB_LAYOUT)?.into_raw();

        // SAFETY: the buddy allocator gave `block` out for this slab, so
        // it holds no slab in use.
        Ok(unsafe {
            block
                .as_ref()
                .unused_slab_descriptor_mut()
                .assign(SlabDescriptor::new())
        })
    }
}

// SAFETY: a layout always picks the same generic size, so a block returns
// to the cache it came from.
unsafe impl<S: SlabSystem> Allocator for SlabAllocator<S> {
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
