#![no_std]
#![feature(ascii_char)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(ascii_char_variants)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(unsafe_cell_access)]
#![allow(static_mut_refs)]
pub mod advanced_writer;
pub mod color_code;
pub mod generic_writer;
pub mod screen_char;
pub mod writer;

use color_code::ColorCode;

use core::fmt::{self, Write};

static mut WRITER: LateInit<Writer<80, 25>> =
    LateInit::new(Writer::default());

pub fn vga_print(args: fmt::Arguments<'_>, color: Option<ColorCode>) {
    unsafe {
        WRITER.inner.set_color(color);

        WRITER.write_fmt(args).unwrap();

        WRITER.inner.set_color(Some(ColorCode::default()));
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

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use ::core::fmt::Write;
    use core::cell::UnsafeCell;
    use std::ascii::Char;
    use std::boxed::Box;
    use std::vec::Vec;

    use crate::{
        advanced_writer::AdvancedWriter,
        color_code::ColorCode,
        generic_writer::{GenericWriter, Writer},
        screen_char::ScreenChar,
    };

    static mut backing: UnsafeCell<[ScreenChar; 80 * 25]> =
        UnsafeCell::new([ScreenChar::default(); 80 * 25]);

    #[test]
    fn it_works() {
        let mut writer =
            UnsafeCell::new(AdvancedWriter::<80, 25>::default());
        unsafe {
            writer.as_mut_unchecked().backing = backing.as_mut_unchecked();
        }

        let mut gen_writer =
            Writer::new(unsafe { writer.as_mut_unchecked() });

        gen_writer.write_str("Hello, World!").unwrap();
        gen_writer.write_str("Hello, World!").unwrap();

        std::println!("Screen Start: \n{:?}", unsafe {
            writer.as_ref_unchecked().screen_start
        });

        std::println!("Cursor: \n{:?}", unsafe {
            writer.as_ref_unchecked().cursor
        });

        gen_writer.inner.update();
        let b = unsafe { backing.as_ref_unchecked() };

        for char in 0..2000 {
            if char % 80 == 0 && char != 0 {
                std::println!("|");
            }
            if b[char].char == Char::LineFeed {
                continue;
            }

            std::print!("{}", unsafe { b[char].char });
        }
        std::println!();
    }
}
