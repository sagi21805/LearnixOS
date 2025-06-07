mod allocator;
mod extension_traits;

use allocator::PageAllocator;
use common::constants::addresses::PAGE_ALLOCATOR_OFFSET;
use core::mem::MaybeUninit;
use cpu_utils::structures::paging::address_types::PhysicalAddress;

pub static mut ALLOCATOR: MaybeUninit<PageAllocator> = unsafe {
    MaybeUninit::new(PageAllocator::new(
        PhysicalAddress(PAGE_ALLOCATOR_OFFSET),
        0x100000000,
    ))
};
