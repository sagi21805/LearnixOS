pub mod color_code;
mod screen_char;
mod writer;

use color_code::ColorCode;
use writer::Writer;

#[allow(private_interfaces)]
pub static mut WRITER: Writer = Writer::default();
pub const SCREEN_WIDTH: usize = 80;
pub const SCREEN_HEIGHT: usize = 25;
use core::fmt::{self, Write};

pub fn vga_print(color: Option<ColorCode>, args: fmt::Arguments<'_>) {
    unsafe {
        if let Some(c) = color {
            WRITER.color = c;
        }

        WRITER.write_fmt(args).unwrap();

        if color.is_some() {
            WRITER.color = ColorCode::default();
        }
    }
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
        $crate::drivers::vga_display::vga_print(None, format_args!($fmt, $($arg,)*))
    }};

    // Case 2: Print with custom color.
    ($fmt:expr $(, $arg:expr)* ; color = $color:expr) => {{
        $crate::drivers::vga_display::vga_print(Some($color), format_args!($fmt, $($arg,)*))
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
macro_rules! fail_msg {
    // Case 1: Print "FAIL" with formatted message.
    ($fmt:expr $(, $arg:tt)*) => {{
        use $crate::drivers::vga_display::color_code::ColorCode;
        $crate::print!("[");
        $crate::print!("FAIL" ; color = ColorCode::new(Color::Red, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    // Case 2: Print "FAIL" with custom message color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        use $crate::drivers::vga_display::color_code::ColorCode;
        $crate::print!("[");
        $crate::print!("FAIL" ; color = ColorCode::new(Color::Red, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }};
}

/// Prints a standardized success message in green color
/// with optional formatting and message color.
#[macro_export]
macro_rules! ok_msg {
    // Case 1: Print "OK" with formatted message.
    ($fmt:expr $(, $arg:tt)*) => {{
        use $crate::drivers::vga_display::color_code::ColorCode;
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new(Color::Green, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    // Case 2: Print "OK" with custom message color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        use $crate::drivers::vga_display::color_code::ColorCode;
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new(Color::Green, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }};
}

/// Clears the VGA screen using the current writer instance.
#[macro_export]
macro_rules! clear {
    () => {{
        use core::fmt::Write;
        use $crate::screen::WRITER;
        use $crate::screen::color_code::ColorCode;
        WRITER.clear()
    }};
}
