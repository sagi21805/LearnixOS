#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]

mod drivers;
mod memory;

use common::constants::addresses::{MEMORY_MAP_LENGTH, MEMORY_MAP_OFFSET};
use common::constants::enums::MemoryRegionType;
use core::arch::asm;
use core::panic::PanicInfo;
use drivers::vga_display::color_code::Color;
use memory::allocators::page_allocator::ALLOCATOR;

use crate::memory::memory_map::MemoryRegionExtended;

/// Kernel entry point for initializing memory management and reporting memory regions.
///
/// This function is the first code executed after entering long mode. It sets up segment registers,
/// prints status messages, initializes the global page allocator, and parses the system memory map
/// from fixed memory addresses. It reports each memory region and accumulates totals for usable and
/// reserved memory, printing a summary before halting execution in an infinite loop.
///
/// # Safety
///
/// This function must be called only as the initial entry point after boot, with all required
/// memory map and environment setup already performed. It assumes the presence of a valid memory
/// map at predefined addresses and does not return.
///
/// # Examples
///
/// ```no_run
/// // This function is not intended to be called directly from Rust code.
/// // It is invoked by the bootloader as the kernel entry point.
/// ```
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
    let mut usable = 0u64;
    let mut reserved = 0u64;
    let length = *(MEMORY_MAP_LENGTH as *const u32) as u16;
    for i in 0..length {
        unsafe {
            let e = (*((MEMORY_MAP_OFFSET + i * 0x24) as *const MemoryRegionExtended)).clone();
            println!("{:x?}\n", e);
            match e.region_type {
                MemoryRegionType::Reserved => reserved += e.length,
                MemoryRegionType::Usable => usable += e.length,
                _ => {}
            }
        }
    }
    let total = usable + reserved;
    println!(
        "Usable Mem: {}, Reserved Mem: {} Total Mem: {}",
        usable, reserved, total
    );

    loop {}
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    fail_msg!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}
