#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]

mod allocators;
mod drivers;

use allocators::page_allocator::ALLOCATOR;
use common::constants::enums::PageSize;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::arch::x86_64::_mm_crc32_u8;
use core::panic::PanicInfo;
use cpu_utils::registers::cr3::{cr3_read, get_current_page_table};
use cpu_utils::structures::paging::page_tables::{PageTable, PageTableEntry};
use drivers::vga_display::color_code::Color;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn _start() -> ! {
    asm!(
        "mov {0}, 0x10",
        "mov ds, {0}",
        "mov es, {0}",
        "mov ss, {0}",
        out(reg) _
    );
    ok_msg!("Entered Protected Mode");
    ok_msg!("Enabled Paging");
    ok_msg!("Entered Long Mode");
    let _ = ALLOCATOR.assume_init_mut().init();
    ok_msg!("Allocator Initialized");
    loop {}
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    fail_msg!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}
