pub mod allocator;
pub (in self) mod paging_extension;
pub (in self) mod address_type_extension;

use allocator::PageAllocator;

const PAGE_ALLOCATOR_OFFSET: usize = 0x0;

pub static mut PAGE_ALLOCATOR: *mut PageAllocator = PAGE_ALLOCATOR_OFFSET as *mut PageAllocator;