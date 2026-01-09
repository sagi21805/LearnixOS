pub mod allocator;
pub mod buddy;
pub mod extensions;

#[macro_export]
/// Allocate the amount of pages specified, and return the address
macro_rules! alloc_pages {
    ($page_number: expr) => {{
        use $crate::memory::allocators::page_allocator::buddy::BUDDY_ALLOCATOR;
        BUDDY_ALLOCATOR.alloc_pages($page_number)
    }};
}
