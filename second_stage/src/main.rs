#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]

mod constants;
mod mbr;
// mod screenEx;
mod screen;

use core::{arch::asm, panic::PanicInfo};
use enums::Color::{self, *};
use mbr::MasterBootRecord;
use screen::{ColorCode, WRITER, ColorAble};
use core::fmt::Write;


#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn _start() -> ! {
        
    let mbr: &MasterBootRecord = unsafe {
        core::mem::transmute((0x7c00 + 446) as *const MasterBootRecord)
    };
    for entry in &mbr.entries {
        println!("{}", entry.relative_sector);
    }
    loop {}
}


/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    print!("[");
    print!("{}", "FAIL".color(ColorCode::new(Red, Black)));
    print!("]: ");
    print!("{}", _info);
    loop {}
}
