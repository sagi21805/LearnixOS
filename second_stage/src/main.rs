#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]

use constants::enums::Color;
use core::panic::PanicInfo;
use utils::{
    print, println,
    screen::color_code::ColorCode
};


#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn _start() -> ! {
    let dn = core::ptr::read((0x7c00 - 2) as *const u16);
    println!("Display Something");
    println!("Stack Pointer: 0x{:x}", dn ; color = ColorCode::new(Color::Yellow, Color::Black));
    println!("size: {}", (size_of::<usize>()));
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
