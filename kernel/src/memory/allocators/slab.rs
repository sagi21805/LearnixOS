pub mod cache;
pub mod descriptor;
pub mod macros;
pub mod traits;

use crate::{
    define_slab_system,
    memory::{
        allocators::slab::{
            cache::SlabCache, descriptor::SlabDescriptor,
            traits::SlabPosition,
        },
        page_descriptor::Unassigned,
    },
};
use core::ptr::NonNull;

// Global Slabs Array Definition
define_slab_system!(SlabDescriptor<Unassigned>,);

pub fn slab_of<T: SlabPosition>() -> &'static mut SlabCache<T> {
    unsafe { SLABS[T::POSITION].assign_mut() }
}

// Marker Extensions
use extend::ext;
#[ext]
impl NonNull<SlabDescriptor<Unassigned>> {
    fn assign<T: SlabPosition>(self) -> NonNull<SlabDescriptor<T>> {
        unsafe { self.as_ref().assign::<T>() }
    }
}

// Thread safety implementations
unsafe impl<T: SlabPosition> Send for SlabDescriptor<T> {}
unsafe impl<T: SlabPosition> Sync for SlabDescriptor<T> {}
unsafe impl<T: SlabPosition> Send for SlabCache<T> {}
unsafe impl<T: SlabPosition> Sync for SlabCache<T> {}
