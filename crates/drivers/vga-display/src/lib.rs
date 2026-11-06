#![no_std]
#![feature(core_io)]
#![feature(ascii_char)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(ascii_char_variants)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
pub mod advanced_writer;
pub mod ansi;
pub mod color_code;
pub mod console;
pub mod generic_writer;
pub mod screen_char;
pub mod writer;

use core::{fmt, io::Write};

use crate::{
    advanced_writer::AdvancedWriter, generic_writer::Writer,
    writer::SimpleWriter,
};

use sync::{mutex::SpinMutex, once::Once};

pub use console::{CONSOLE, Console};

/// The writer that needs no heap, used by [`WRITER`] until a better one is
/// set, and by the panic handler.
// SAFETY: only `WRITER`'s `leak` and the panic handler's `force_unlock`
// reach it.
pub static SIMPLE_WRITER: SpinMutex<SimpleWriter<80, 25>> =
    unsafe { SpinMutex::new_locked(SimpleWriter::default()) };

/// The writer behind [`print!`] and [`println!`].
// SAFETY: this is the only `leak` of `SIMPLE_WRITER`.
pub static WRITER: SpinMutex<Writer<'static>> =
    SpinMutex::new(Writer::new(unsafe { SIMPLE_WRITER.leak() }));

/// The writer with a scrollback buffer, which replaces [`SIMPLE_WRITER`]
/// in [`WRITER`] once the heap works.
pub static ADVANCED_WRITER: Once<SpinMutex<AdvancedWriter<80, 25>>> =
    Once::uninit();

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
