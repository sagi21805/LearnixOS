#![no_std]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(ptr_alignment_type)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

pub mod cache;
pub mod cache_allocator;
pub mod descriptor;
pub mod local_macros;
pub mod preallocated;
pub mod slab_address;
pub mod slab_objects;
pub mod traits;

use crate::{
    cache::SlabCache,
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

pub struct SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    slab_arena: &'static SpinMutex<[SlabCache<()>; COUNT]>,
    buddy_arena: &'static SpinMutex<Arena>,
    // Wrap in a mutex to automatically implement Sync and Send.
    _block: PhantomData<SpinMutex<Block>>,
}

define_slab_system!(
    Generic8,
    Generic16,
    Generic32,
    Generic64,
    Generic96,
    Generic128,
    Generic192,
    Generic256,
    Generic512,
    Generic1024,
    Generic2048,
    Generic4096,
    Generic8192,
);

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
        // SAFETY: the caller guarantees `ptr` is a live object from this
        // cache, allocated with `Layout::new::<T>()`.
        unsafe {
            self.cache::<T>().deallocate(ptr.cast(), Layout::new::<T>())
        }
    }
}
