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

use alloc::boxed::Box;
use core::fmt;

use console::{Console, SCREEN, WIDTH};
use row_history::RowHistory;
pub use scrollback::Scrollback;

/// Prints on the screen, which is what `libk`'s print macros reach at link
/// time.
#[unsafe(no_mangle)]
pub fn kprint(args: fmt::Arguments<'_>) {
    Console::new(&mut SCREEN.lock()).print(args);
}

/// Starts keeping finished rows, beginning with those above the cursor.
///
/// A second call drops the rows the first scrollback kept.
pub fn attach_scrollback(scrollback: Box<Scrollback>) {
    let scrollback = Box::leak(scrollback);
    let mut screen = SCREEN.lock();
    let cursor_row = screen.cursor / WIDTH;
    for row in &screen.vga.rows()[..cursor_row] {
        scrollback.push_row(row);
    }
    screen.history = scrollback;
}

/// Moves the view `lines` rows up. Until [`attach_scrollback`], no rows
/// are kept, so this does nothing.
pub fn scroll_view_up(lines: usize) {
    Console::new(&mut SCREEN.lock()).scroll_view_up(lines);
}

/// Moves the view `lines` rows down, stopping at the output area.
pub fn scroll_view_down(lines: usize) {
    Console::new(&mut SCREEN.lock()).scroll_view_down(lines);
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
    Console::new(&mut SCREEN.lock()).panic_message(message);
}
