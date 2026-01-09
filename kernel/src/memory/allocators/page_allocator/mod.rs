pub mod allocator;
pub mod extensions;

use allocator::PhysicalPageAllocator;
use core::mem::MaybeUninit;

pub static mut ALLOCATOR: MaybeUninit<PhysicalPageAllocator> =
    MaybeUninit::uninit();

#[macro_export]
/// Allocate the amout of pages specified, and return the address
macro_rules! alloc_pages {
    ($page_number: expr) => {{
        use core::alloc::{Allocator, Layout};
        use $crate::memory::allocators::page_allocator::ALLOCATOR;
        ALLOCATOR
            .assume_init_ref()
            .allocate(Layout::from_size_align_unchecked(
                REGULAR_PAGE_SIZE * $page_number,
                REGULAR_PAGE_ALIGNMENT.as_usize(),
            ))
            .unwrap()
            .addr()
            .get()
    }};
}
