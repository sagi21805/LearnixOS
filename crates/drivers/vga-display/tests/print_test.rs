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

use common::late_init::LateInit;
use vga_display::{
    advanced_writer::AdvancedWriter,
    generic_writer::{GenericWriter, Writer},
    screen_char::ScreenChar,
};

static mut BACKING: UnsafeCell<[ScreenChar; 80 * 25]> =
    UnsafeCell::new([ScreenChar::default(); 80 * 25]);

static mut ADVANCED_WRITER: LateInit<AdvancedWriter<80, 25>> =
    LateInit::uninit();

static mut WRITER: Writer =
    Writer::new(unsafe { ADVANCED_WRITER.assume_init_mut() });

fn update(writer: &'static mut Writer) {
    let b = unsafe { BACKING.as_ref_unchecked() };
    std::thread::spawn(move || {
        loop {
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
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

#[test]
fn it_works() {
    unsafe {
        ADVANCED_WRITER
            .write(unsafe { AdvancedWriter::<80, 25>::default() });
        ADVANCED_WRITER.assume_init_mut().backing =
            BACKING.as_mut_unchecked();
    }

    update(unsafe { &mut WRITER });

    for i in 0..50 {
        unsafe {
            WRITER.write_fmt(format_args!("{}\n", i)).unwrap();
        }
    }

    // gen_writer
    //     .write_fmt(format_args!("{:?}", unsafe {
    //         writer.as_ref_unchecked()
    //     }))
    //     .unwrap();

    std::println!("Screen Start: \n{:?}", unsafe {
        unsafe { ADVANCED_WRITER.screen_start.get() }
    });

    std::println!("Cursor: \n{:?}", unsafe {
        unsafe { ADVANCED_WRITER.cursor.get() }
    });
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
