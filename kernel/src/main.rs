#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]

use constants::enums::Color;
use utils::{println, print};
use core::panic::PanicInfo;
use core::arch::asm;

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
    println!("This is a call from x64");
    
    loop {}
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    print!("[");
    print!("FAIL" ; color = ColorCode::new(Color::Red, Color::Black));
    print!("]: ");
    println!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}