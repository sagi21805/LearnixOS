#![no_std]
#![feature(core_io)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
pub mod ansi;
pub mod color_code;
pub mod console;
pub mod screen_char;

use core::{fmt, io::Write};

pub use console::{CONSOLE, Console};

/// Prints through [`CONSOLE`], which is what `libk`'s print macros reach
/// at link time.
#[unsafe(no_mangle)]
pub fn kprint(args: fmt::Arguments<'_>) {
    let mut console = CONSOLE.lock();
    console
        .write_fmt(args)
        .expect("only a failing Display implementation fails a print");
    console.flush().expect("flushing the console can't fail");
}
