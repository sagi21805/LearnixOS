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

use console::{Console, SCREEN};
use row_history::{NoHistory, RowHistory};
pub use scrollback::Scrollback;
use sync::{mutex::SpinMutex, once::Once};

/// The scrollback, empty until the heap exists.
static HISTORY: Once<SpinMutex<Scrollback>> = Once::uninit();

/// Prints on the screen, which is what `libk`'s print macros reach at link
/// time.
#[unsafe(no_mangle)]
pub fn kprint(args: fmt::Arguments<'_>) {
    let mut screen = SCREEN.lock();
    match HISTORY.get() {
        Some(scrollback) => {
            Console::new(&mut screen, &mut *scrollback.lock()).print(args)
        }
        None => Console::new(&mut screen, &mut NoHistory).print(args),
    }
}

/// Starts keeping finished rows, beginning with those above the cursor.
///
/// # Panics
///
/// When a scrollback is already attached.
pub fn attach_scrollback(mut scrollback: Scrollback) -> ScrollControl {
    let mut screen = SCREEN.lock();
    for row in screen.finished_rows() {
        scrollback.push_row(row);
    }
    ScrollControl {
        scrollback: HISTORY.init(SpinMutex::new(scrollback)),
    }
}

/// Scrolls the console's view, which only [`attach_scrollback`] can hand
/// out.
#[derive(Clone, Copy)]
pub struct ScrollControl {
    scrollback: &'static SpinMutex<Scrollback>,
}

impl ScrollControl {
    /// Moves the view `lines` rows up.
    pub fn scroll_view_up(self, lines: usize) {
        let mut screen = SCREEN.lock();
        let mut scrollback = self.scrollback.lock();
        Console::new(&mut screen, &mut *scrollback).scroll_view_up(lines);
    }

    /// Moves the view `lines` rows down.
    pub fn scroll_view_down(self, lines: usize) {
        let mut screen = SCREEN.lock();
        let mut scrollback = self.scrollback.lock();
        Console::new(&mut screen, &mut *scrollback)
            .scroll_view_down(lines);
    }
}

/// Prints a panic message, whatever state the panic left the console in.
///
/// # Safety
///
/// Only the panic handler may call this, as it breaks locks that a
/// panicking holder will never release.
pub unsafe fn print_panic_message(message: fmt::Arguments<'_>) {
    unsafe { SCREEN.force_unlock() };
    let mut screen = SCREEN.lock();
    match HISTORY.get() {
        Some(scrollback) => {
            unsafe { scrollback.force_unlock() };
            Console::new(&mut screen, &mut *scrollback.lock())
                .panic_message(message)
        }
        None => Console::new(&mut screen, &mut NoHistory)
            .panic_message(message),
    }
}
