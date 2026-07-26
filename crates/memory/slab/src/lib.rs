#![no_std]
#![feature(specialization)]
#![feature(allocator_api)]
#![feature(ptr_alignment_type)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

pub mod cache;
pub mod descriptor;
pub mod local_macros;
pub mod traits;

use crate::{
    cache::SlabCache,
    traits::{Generic, Slab, SlabBlock, SlabPosition},
};
use core::{
    alloc::{AllocError, Allocator},
    marker::PhantomData,
    ptr::NonNull,
};

use common::address_types::{Address, VirtualAddress};
use sync::mutex::SpinMutex;

use buddy::meta::{BuddyArena, BuddyBlock};

use macros::generate_generics;
use x86::structures::paging::VirtualAddressExt;

generate_generics!(
    8, 16, 32, 64, 96, 128, 192, 256, 512, 1024, 2048, 4096, 8192
);

pub struct SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    slabs: [SlabCache<()>; COUNT],
    arena: &'static SpinMutex<Arena>,
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
    pub const fn new(arena: &'static SpinMutex<Arena>) -> Self {
        const EMPTY_SLAB: SlabCache<()> = SlabCache::default();
        Self {
            slabs: [EMPTY_SLAB; COUNT],
            arena,
            _block: PhantomData,
        }
    }

    pub fn slab_of<T: Slab>(&self) -> NonNull<SlabCache<T>> {
        NonNull::from_ref(&self.slabs[T::SLAB_POSITION]).cast()
    }

    pub fn kmalloc<T: Slab>(&self) -> NonNull<T> {
        let mut slab = self.slab_of::<T>();
        unsafe { slab.as_mut().alloc() }
    }

    pub fn kfree<T: Slab>(&self, ptr: NonNull<T>) {
        let mut page = self
            .arena
            .lock()
            .page_with_address(unsafe {
                VirtualAddress::new_unchecked(ptr.addr().get())
                    .translate()
                    .unwrap()
            })
            .unwrap();

        let descriptor =
            unsafe { page.as_mut().slab_descriptor_mut::<T>() };

        unsafe { descriptor.dealloc(ptr) };
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
                Ok(self.kmalloc::<Generic8>().into_u8())
            }
            Generic16::START..=Generic16::END => {
                Ok(self.kmalloc::<Generic16>().into_u8())
            }
            Generic32::START..=Generic32::END => {
                Ok(self.kmalloc::<Generic32>().into_u8())
            }
            Generic64::START..=Generic64::END => {
                Ok(self.kmalloc::<Generic64>().into_u8())
            }
            Generic96::START..=Generic96::END => {
                Ok(self.kmalloc::<Generic96>().into_u8())
            }
            Generic128::START..=Generic128::END => {
                Ok(self.kmalloc::<Generic128>().into_u8())
            }
            Generic192::START..=Generic192::END => {
                Ok(self.kmalloc::<Generic192>().into_u8())
            }
            Generic256::START..=Generic256::END => {
                Ok(self.kmalloc::<Generic256>().into_u8())
            }
            Generic512::START..=Generic512::END => {
                Ok(self.kmalloc::<Generic512>().into_u8())
            }
            Generic1024::START..=Generic1024::END => {
                Ok(self.kmalloc::<Generic1024>().into_u8())
            }
            Generic2048::START..=Generic2048::END => {
                Ok(self.kmalloc::<Generic2048>().into_u8())
            }
            Generic4096::START..=Generic4096::END => {
                Ok(self.kmalloc::<Generic4096>().into_u8())
            }
            Generic8192::START..=Generic8192::END => {
                Ok(self.kmalloc::<Generic8192>().into_u8())
            }
            _ => Err(AllocError),
        }
    }

    unsafe fn deallocate(
        &self,
        ptr: core::ptr::NonNull<u8>,
        layout: core::alloc::Layout,
    ) {
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
