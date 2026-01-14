pub mod cache;
pub mod descriptor;
pub mod macros;
pub mod traits;

use crate::{
    define_slab_system,
    memory::{
        allocators::slab::{
            cache::SlabCache,
            descriptor::SlabDescriptor,
            traits::{SlabCacheConstructor, SlabPosition},
        },
        page_descriptor::Unassigned,
    },
};
use core::ptr::NonNull;

pub struct Generic8(pub usize);
pub struct Generic16(pub [usize; 2]);
pub struct Generic32(pub [usize; 4]);
pub struct Generic64(pub [usize; 8]);
pub struct Generic96(pub [usize; 12]);
pub struct Generic128(pub [usize; 16]);
pub struct Generic192(pub [usize; 24]);
pub struct Generic256(pub [usize; 32]);
pub struct Generic512(pub [usize; 64]);
pub struct Generic1024(pub [usize; 128]);
pub struct Generic2048(pub [usize; 256]);
pub struct Generic4096(pub [usize; 512]);
pub struct Generic8192(pub [usize; 1024]);

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
}

#[extend::ext]
impl NonNull<SlabDescriptor<Unassigned>> {
    fn assign<T: SlabPosition>(self) -> NonNull<SlabDescriptor<T>> {
        unsafe { self.as_ref().assign::<T>() }
    }
}

unsafe impl<T: SlabPosition> Send for SlabDescriptor<T> {}
unsafe impl<T: SlabPosition> Sync for SlabDescriptor<T> {}
unsafe impl<T: SlabPosition> Send for SlabCache<T> {}
unsafe impl<T: SlabPosition> Sync for SlabCache<T> {}
