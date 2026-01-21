pub mod cache;
pub mod descriptor;
pub mod macros;
pub mod traits;

use common::address_types::VirtualAddress;
use learnix_macros::generate_generics;

use crate::{
    define_slab_system,
    memory::{
        allocators::{
            extensions::VirtualAddressExt,
            slab::{
                cache::SlabCache,
                descriptor::SlabDescriptor,
                traits::{SlabCacheConstructor, SlabPosition},
            },
        },
        page_descriptor::{PAGES, Unassigned, UnassignedPage},
    },
};
use core::{
    alloc::{AllocError, Allocator},
    ptr::NonNull,
};

pub trait Generic {
    const START: usize;
    const END: usize;

    fn size(&self) -> usize;
}

generate_generics!(
    8, 16, 32, 64, 96, 128, 192, 256, 512, 1024, 2048, 4096, 8192
);

define_slab_system!(
    SlabDescriptor<Unassigned>,
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

pub static mut SLAB_ALLOCATOR: SlabAllocator = SlabAllocator::new();

impl SlabAllocator {
    pub fn slab_of<T: SlabPosition>(&self) -> NonNull<SlabCache<T>> {
        self.slabs[T::POSITION].assign::<T>()
    }

    pub fn kmalloc<T: SlabPosition>(&self) -> NonNull<T> {
        let mut slab = self.slab_of::<T>();
        unsafe { slab.as_mut().alloc() }
    }

    pub fn kfree<T: SlabPosition>(&self, ptr: NonNull<T>) {
        let index = UnassignedPage::index_of_page(unsafe {
            VirtualAddress::new_unchecked(ptr.as_ptr() as usize)
                .translate()
        });

        let page = unsafe { PAGES[index].assign::<T>().as_ref() };

        if let Some(mut descriptor) = page.owner {
            unsafe { descriptor.as_mut().dealloc(ptr) };
        } else {
            panic!("Object is freed from a page that has not owner!")
        }
    }
}

#[extend::ext]
impl NonNull<SlabDescriptor<Unassigned>> {
    fn assign<T: SlabPosition>(self) -> NonNull<SlabDescriptor<T>> {
        unsafe { self.as_ref().assign::<T>() }
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

unsafe impl Allocator for SlabAllocator {
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
unsafe impl<T: SlabPosition> Send for SlabDescriptor<T> {}
unsafe impl<T: SlabPosition> Sync for SlabDescriptor<T> {}
unsafe impl<T: SlabPosition> Send for SlabCache<T> {}
unsafe impl<T: SlabPosition> Sync for SlabCache<T> {}
