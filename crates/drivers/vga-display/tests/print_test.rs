#![feature(unsafe_cell_access)]
#![feature(ascii_char)]
#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(ascii_char_variants)]
#![allow(static_mut_refs)]

extern crate std;

use ::core::fmt::Write;
use core::cell::UnsafeCell;
use std::ascii::Char;

use vga_display::{
    advanced_writer::AdvancedWriter, generic_writer::Writer,
    screen_char::ScreenChar,
};

static mut backing: UnsafeCell<[ScreenChar; 80 * 25]> =
    UnsafeCell::new([ScreenChar::default(); 80 * 25]);

#[test]
fn it_works() {
    let writer = UnsafeCell::new(AdvancedWriter::<80, 25>::default());
    unsafe {
        writer.as_mut_unchecked().backing = backing.as_mut_unchecked();
    }

    let mut gen_writer = Writer::new(unsafe { writer.as_mut_unchecked() });

    gen_writer.write_str("Hello, World!").unwrap();
    gen_writer.write_str("Hello, World!").unwrap();

    std::println!("Screen Start: \n{:?}", unsafe {
        writer.as_ref_unchecked().screen_start.get()
    });

    std::println!("Cursor: \n{:?}", unsafe {
        writer.as_ref_unchecked().cursor.get()
    });

    gen_writer.inner.update();
    let b = unsafe { backing.as_ref_unchecked() };

    for char in 0..2000 {
        if char % 80 == 0 && char != 0 {
            std::println!("|");
        }
        if b[char].char == Char::LineFeed {
            continue;
        }

        std::print!("{}", unsafe { b[char].char });
    }
    std::println!();
}
