pub mod allocator;
pub mod extension_traits;

use allocator::PhysicalPageAllocator;
use common::constants::addresses::PAGE_ALLOCATOR_OFFSET;
use core::mem::MaybeUninit;
use cpu_utils::structures::paging::address_types::PhysicalAddress;

pub static mut ALLOCATOR: MaybeUninit<PhysicalPageAllocator> = MaybeUninit::uninit();
