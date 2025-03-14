#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]

mod constants;
mod enums;
mod mbr;
// mod screenEx;
mod screen;

use core::{arch::asm, panic::PanicInfo};
use enums::Color::*;
use constants::VGA_BUFFER_PTR;
use screen::{ColorCode, ScreenChar, Writer};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub extern "C" fn _start() {
    // unsafe {
    //     asm!(
    //        "mov ax, 0x10",
    //        "mov ds, ax",
    //        "mov dx, 0x3C0",
    //        "mov al, 0x10",
    //        "out dx, al",
    //        "inc dx",
    //        "in al, dx",
    //        "and al, 0x7F",
    //        "out dx, al",
    //     );
    // }
    let mut w = Writer::new();
    w.print(
        "I can print what ever i wanttttt!!!",
        ColorCode::new(Green, Black)
    );
    loop {} 
}

// pub extern "sysv64" fn my_function(x: u64, y: u64) -> u64 {
//     x + y
// }

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // print(_info.message().as_str().unwrap());
    loop {}
}
