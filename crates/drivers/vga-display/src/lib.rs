#![no_std]
#![feature(ascii_char)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(ascii_char_variants)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
// pub mod advanced_writer;
pub mod color_code;
pub mod generic_writer;
pub mod screen_char;
pub mod writer;

use color_code::ColorCode;
use common::{constants::VGA_BUFFER_PTR, late_init::LateInit};

use core::fmt::{self, Write};

use crate::{generic_writer::Writer, screen_char::ScreenChar};

use sync::mutex::SpinMutex;

pub static SCREEN_LOCK: LateInit<SpinMutex<&'static mut [ScreenChar]>> =
    LateInit::uninit();

unsafe extern "Rust" {
    static WRITER: LateInit<Writer<'static>>;
}

pub fn vga_init() {
    SCREEN_LOCK.init(SpinMutex::new(unsafe {
        core::slice::from_raw_parts_mut(
            VGA_BUFFER_PTR as *mut ScreenChar,
            80 * 25,
        )
    }));
}

pub fn vga_print(args: fmt::Arguments<'_>, color: Option<ColorCode>) {
    unsafe {
        WRITER.inner.lock().set_color(color.unwrap_or_default());

        WRITER.inner.lock().write_fmt(args).unwrap();

        WRITER.inner.lock().set_color(ColorCode::default());
    }
}
#[unsafe(no_mangle)]
pub fn kprint(args: fmt::Arguments<'_>) {
    vga_print(args, None);
}

/// Prints formatted text to the VGA display without a
/// newline.
///
/// # Parameters
/// - `$fmt`: The format string.
/// - `$arg`: Optional arguments to interpolate into the format string.
/// - `color = $color`: Optional named parameter to change the VGA text
///   color for this print.
#[macro_export]
macro_rules! print {
    // Case 1: Standard print with optional arguments.
    ($fmt:expr $(, $arg:expr)* $(;)?) => {{
        vga_display::vga_print(format_args!($fmt, $($arg,)*), None)
    }};

    // Case 2: Print with custom color.
    ($fmt:expr $(, $arg:expr)* ; color = $color:expr) => {{
        vga_display::vga_print(format_args!($fmt, $($arg,)*), Some($color))
    }};
}

/// Prints formatted text followed by a newline to the VGA
/// display. Same as the [`print!`] macro just with a `\n`
/// attached to the end
#[macro_export]
macro_rules! println {
    // Case 1: Standard println with optional arguments.
    ($fmt:expr $(, $arg:expr)* $(;)?) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)*)
    };
    // Case 2: println with custom color.
    ($fmt:expr $(, $arg:expr)* ; color = $color:expr) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)* ; color = $color)
    };
}

/// Prints a standardized failure message in red color with
/// optional formatting and message color.
#[macro_export]
macro_rules! eprintln {
    // Case 1: Print "FAIL" with formatted message.
    ($fmt:expr $(, $arg:tt)*) => {{
        use $crate::drivers::vga_display::color_code::ColorCode;
        use common::enums::Color;
        $crate::print!("[");
        $crate::print!("FAIL" ; color = ColorCode::new().foreground(Color::Red).background(Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    // Case 2: Print "FAIL" with custom message color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        use vga_display::color_code::ColorCode;
        use common::enums::Color;
        $crate::print!("[");
        $crate::print!("FAIL" ; color = ColorCode::new().foreground(Color::Red).background(Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }};
}

/// Prints a standardized success message in green color
/// with optional formatting and message color.
#[macro_export]
macro_rules! okprintln {
    // Case 1: Print "OK" with formatted message.
    ($fmt:expr $(, $arg:tt)*) => {{
        use vga_display::color_code::ColorCode;
        use common::enums::Color;
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new().foreground(Color::Green).background(Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    // Case 2: Print "OK" with custom message color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        use $crate::drivers::vga_display::color_code::ColorCode;
        use common::enums::Color;
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new().foreground(Color::Green).background(Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }};
}
