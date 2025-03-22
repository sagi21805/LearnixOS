#![allow(unsafe_op_in_unsafe_fn)]

pub mod color_code;
mod screen_char;
mod writer;

use color_code::ColorCode;
use writer::Writer;

pub static mut WRITER: Writer = Writer::new(ColorCode::default());
static SCREEN_WIDTH: usize = 80;
static SCREEN_HEIGHT: usize = 25;

#[macro_export]
macro_rules! print {
    // Case 1: Format + args*
    ($fmt:expr $(, $arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::screen::WRITER;
        unsafe {
            write!(WRITER, $fmt, $($arg)*).unwrap();
            WRITER.color = ColorCode::default();
        }
    }};

    // Case 2: Format + args* + color
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        use core::fmt::Write;
        use $crate::screen::WRITER;
        unsafe {
            WRITER.color = $color;
            write!(WRITER, $fmt, $($arg)*).unwrap();
            WRITER.color = ColorCode::default();
        }
    }};
}

#[macro_export]
macro_rules! println {
    // Case 1: Format + args* + color
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)* ; color = $color)
    };

    // Case 2: Format + args*
    ($fmt:expr $(, $arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)*)
    };
}
