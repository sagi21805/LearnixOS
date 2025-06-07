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
