#![no_std]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(allocator_ext)]
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
    traits::{Generic, Slab, SlabBlock, SlabPosition},
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

#[extend::ext]
pub impl<T: Generic> NonNull<T> {
    fn into_u8(&self) -> NonNull<[u8]> {
        unsafe {
            let data = NonNull::new_unchecked(self.as_ptr() as *mut u8);
            let size = self.as_ref().size();
            NonNull::slice_from_raw_parts(data, size)
        }
    }

    fn from_u8(data: NonNull<u8>) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(data.as_ptr() as *mut T) }
    }
}

unsafe impl<Block, Arena> Allocator for SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        if layout.size() < layout.align() {
            return Err(AllocError);
        }

        match layout.size() {
            Generic8::START..=Generic8::END => {
                self.kmalloc::<Generic8>().map(|object| object.into_u8())
            }
            Generic16::START..=Generic16::END => {
                self.kmalloc::<Generic16>().map(|object| object.into_u8())
            }
            Generic32::START..=Generic32::END => {
                self.kmalloc::<Generic32>().map(|object| object.into_u8())
            }
            Generic64::START..=Generic64::END => {
                self.kmalloc::<Generic64>().map(|object| object.into_u8())
            }
            Generic96::START..=Generic96::END => {
                self.kmalloc::<Generic96>().map(|object| object.into_u8())
            }
            Generic128::START..=Generic128::END => {
                self.kmalloc::<Generic128>().map(|object| object.into_u8())
            }
            Generic192::START..=Generic192::END => {
                self.kmalloc::<Generic192>().map(|object| object.into_u8())
            }
            Generic256::START..=Generic256::END => {
                self.kmalloc::<Generic256>().map(|object| object.into_u8())
            }
            Generic512::START..=Generic512::END => {
                self.kmalloc::<Generic512>().map(|object| object.into_u8())
            }
            Generic1024::START..=Generic1024::END => self
                .kmalloc::<Generic1024>()
                .map(|object| object.into_u8()),
            Generic2048::START..=Generic2048::END => self
                .kmalloc::<Generic2048>()
                .map(|object| object.into_u8()),
            Generic4096::START..=Generic4096::END => self
                .kmalloc::<Generic4096>()
                .map(|object| object.into_u8()),
            Generic8192::START..=Generic8192::END => self
                .kmalloc::<Generic8192>()
                .map(|object| object.into_u8()),
            _ => Err(AllocError),
        }
    }

    unsafe fn deallocate(
        &self,
        ptr: core::ptr::NonNull<u8>,
        layout: core::alloc::Layout,
    ) {
        // SAFETY: the caller guarantees `ptr` came from `allocate` with
        // this layout, which picked the same size class.
        unsafe {
            match layout.size() {
                Generic8::START..=Generic8::END => {
                    self.kfree::<Generic8>(NonNull::from_u8(ptr))
                }
                Generic16::START..=Generic16::END => {
                    self.kfree::<Generic16>(NonNull::from_u8(ptr))
                }
                Generic32::START..=Generic32::END => {
                    self.kfree::<Generic32>(NonNull::from_u8(ptr))
                }
                Generic64::START..=Generic64::END => {
                    self.kfree::<Generic64>(NonNull::from_u8(ptr))
                }
                Generic96::START..=Generic96::END => {
                    self.kfree::<Generic96>(NonNull::from_u8(ptr))
                }
                Generic128::START..=Generic128::END => {
                    self.kfree::<Generic128>(NonNull::from_u8(ptr))
                }
                Generic192::START..=Generic192::END => {
                    self.kfree::<Generic192>(NonNull::from_u8(ptr))
                }
                Generic256::START..=Generic256::END => {
                    self.kfree::<Generic256>(NonNull::from_u8(ptr))
                }
                Generic512::START..=Generic512::END => {
                    self.kfree::<Generic512>(NonNull::from_u8(ptr))
                }
                Generic1024::START..=Generic1024::END => {
                    self.kfree::<Generic1024>(NonNull::from_u8(ptr))
                }
                Generic2048::START..=Generic2048::END => {
                    self.kfree::<Generic2048>(NonNull::from_u8(ptr))
                }
                Generic4096::START..=Generic4096::END => {
                    self.kfree::<Generic4096>(NonNull::from_u8(ptr))
                }
                Generic8192::START..=Generic8192::END => {
                    self.kfree::<Generic8192>(NonNull::from_u8(ptr))
                }
                _ => unreachable!(),
            }
        }
    }
}
