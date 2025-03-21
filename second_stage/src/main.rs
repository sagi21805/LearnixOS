#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]

mod mbr;
// mod screenEx;
mod screen;

use constants::enums::Color;
use core::panic::PanicInfo;
use screen::{color_code::ColorCode, WRITER};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn _start() -> ! {
    let dn = core::ptr::read((0x7c00 + 0x1b8) as *const u16);
    println!("Display Something, {:x}", dn ; color = ColorCode::new(Color::Green, Color::Black));

    // WRITER.clear();
    // print!("Hello World");
    loop {}
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    print!("[");
    print!("FAIL" ; color = ColorCode::default());
    print!("]: ");
    println!("{}", _info);
    loop {}
}
