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

use drivers::vga_display::color_code::Color;
use core::arch::asm;
use core::panic::PanicInfo;

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
    ok_msg!("Enabled Pageing");
    ok_msg!("Entered Long Mode");
    loop {}
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    fail_msg!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}
