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

static mut BACKING: UnsafeCell<[ScreenChar; 80 * 25]> =
    UnsafeCell::new([ScreenChar::default(); 80 * 25]);

fn update(writer: &mut Writer) {
    let b = unsafe { BACKING.as_ref_unchecked() };

    for char in 0..2000 {
        if char % 80 == 0 && char != 0 {
            std::println!("|");
        }
        if b[char].char == Char::LineFeed {
            continue;
        }

        std::print!("{}", b[char].char);
    }
    std::println!();

    writer.inner.update();
}

#[test]
fn it_works() {
    let writer = UnsafeCell::new(AdvancedWriter::<80, 25>::default());
    unsafe {
        writer.as_mut_unchecked().backing = BACKING.as_mut_unchecked();
    }

    let mut gen_writer = Writer::new(unsafe { writer.as_mut_unchecked() });

    for i in 0..50 {
        gen_writer.write_fmt(format_args!("{}\n", i)).unwrap();
    }

    // gen_writer
    //     .write_fmt(format_args!("{:?}", unsafe {
    //         writer.as_ref_unchecked()
    //     }))
    //     .unwrap();

    std::println!("Screen Start: \n{:?}", unsafe {
        writer.as_ref_unchecked().screen_start.get()
    });

    std::println!("Cursor: \n{:?}", unsafe {
        writer.as_ref_unchecked().cursor.get()
    });
    loop {
        update(&mut gen_writer);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
