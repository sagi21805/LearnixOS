#![no_std]
#![feature(core_io)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(slice_range)]
extern crate alloc;

pub mod ansi;
pub mod color_code;
pub mod console;
mod row_history;
pub mod screen_char;
pub mod scrollback;
mod vga_memory;

use core::fmt;

pub use console::{SCREEN, Screen};
pub use scrollback::Scrollback;

/// Prints on the screen, which is what `libk`'s print macros reach at link
/// time.
#[unsafe(no_mangle)]
pub fn kprint(args: fmt::Arguments<'_>) { SCREEN.lock().print(args); }

/// Prints on the screen unless something already holds it, for handlers
/// that may run while the screen is locked.
///
/// Returns whether it printed.
pub fn try_kprint(args: fmt::Arguments<'_>) -> bool {
    SCREEN
        .try_lock()
        .map(|mut screen| screen.print(args))
        .is_some()
}

/// Prints a panic message, whatever state the panic left the console in.
///
/// # Safety
///
/// Only the panic handler may call this, as it breaks a lock that a
/// panicking holder will never release.
pub unsafe fn print_panic_message(message: fmt::Arguments<'_>) {
    // SAFETY: the caller is the panic handler, so whoever holds the lock
    // will never run again to release it.
    unsafe { SCREEN.force_unlock() };
    SCREEN.lock().panic_message(message);
}
