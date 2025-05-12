mod allocator;
mod extension_traits;

use allocator::PageAllocator;
use core::mem::MaybeUninit;
use cpu_utils::structures::paging::address_types::PhysicalAddress;
const PAGE_ALLOCATOR_OFFSET: usize = 0x10000;

pub static mut ALLOCATOR: MaybeUninit<PageAllocator> = unsafe {
    MaybeUninit::new(PageAllocator::new(
        PhysicalAddress(PAGE_ALLOCATOR_OFFSET),
        0xffffffff,
    ))
};
