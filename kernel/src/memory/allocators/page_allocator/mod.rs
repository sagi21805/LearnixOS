pub mod allocator;
pub mod extensions;

use allocator::PhysicalPageAllocator;
use core::mem::MaybeUninit;

pub static mut ALLOCATOR: MaybeUninit<PhysicalPageAllocator> =
    MaybeUninit::uninit();
