#![no_std]
#![feature(ptr_alignment_type)]
#![feature(macro_metavar_expr_concat)]

#[macro_use]
pub mod macros;
pub mod address_types;
pub mod bitmap;
pub mod constants;
pub mod enums;
pub mod error;
pub mod ring_buffer;
struct FakeAllocator;

unsafe impl core::alloc::GlobalAlloc for FakeAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe {
            unsafe extern "Rust" {
                unsafe fn fake_alloc_this_doesnt_exist(
                    layout: core::alloc::Layout,
                ) -> *mut u8;
            }
            fake_alloc_this_doesnt_exist(layout)
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe {
            unsafe extern "Rust" {
                unsafe fn fake_dealloc_this_doesnt_exist(
                    ptr: *mut u8,
                    layout: core::alloc::Layout,
                );
            }
            fake_dealloc_this_doesnt_exist(ptr, layout)
        }
    }
}

#[global_allocator]
static ALLOCATOR: FakeAllocator = FakeAllocator;
