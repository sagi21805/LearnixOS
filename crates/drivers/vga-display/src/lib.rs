#![no_std]
#![feature(core_io)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(slice_range)]
pub mod ansi;
pub mod color_code;
pub mod console;
mod row_history;
pub mod screen_char;
pub mod scrollback;
mod vga_memory;

use core::fmt;

use console::{CONSOLE, Console, SCREEN};
use row_history::{NoHistory, RowHistory};
pub use scrollback::Scrollback;

/// Prints on the screen, which is what `libk`'s print macros reach at link
/// time.
#[unsafe(no_mangle)]
pub fn kprint(args: fmt::Arguments<'_>) {
    let mut screen = SCREEN.lock();
    match &mut *CONSOLE.lock() {
        Some(scrollback) => {
            Console::new(&mut screen, scrollback).print(args)
        }
        None => Console::new(&mut screen, &mut NoHistory).print(args),
    }
}

/// Starts keeping finished rows, beginning with those above the cursor.
pub fn attach_scrollback(mut scrollback: Scrollback) {
    let mut screen = SCREEN.lock();
    for row in screen.finished_rows() {
        scrollback.push_row(row);
    }
    *CONSOLE.lock() = Some(scrollback);
}

/// Moves the scrollback's view `lines` rows up.
pub fn scroll_view_up(lines: usize) {
    let mut screen = SCREEN.lock();
    if let Some(scrollback) = &mut *CONSOLE.lock() {
        Console::new(&mut screen, scrollback).scroll_view_up(lines);
    }
}

/// Moves the scrollback's view `lines` rows down.
pub fn scroll_view_down(lines: usize) {
    let mut screen = SCREEN.lock();
    if let Some(scrollback) = &mut *CONSOLE.lock() {
        Console::new(&mut screen, scrollback).scroll_view_down(lines);
    }
}

/// Prints a panic message, whatever state the panic left the console in.
///
/// # Safety
///
/// Only the panic handler may call this, as it breaks locks that a
/// panicking holder will never release.
pub unsafe fn print_panic_message(message: fmt::Arguments<'_>) {
    unsafe {
        SCREEN.force_unlock();
        CONSOLE.force_unlock();
    }
    let mut screen = SCREEN.lock();
    match &mut *CONSOLE.lock() {
        Some(scrollback) => {
            Console::new(&mut screen, scrollback).panic_message(message)
        }
        None => Console::new(&mut screen, &mut NoHistory)
            .panic_message(message),
    }
}
