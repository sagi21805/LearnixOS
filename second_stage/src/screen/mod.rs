#![allow(unsafe_op_in_unsafe_fn)]
pub mod color_code;
pub mod screen_char;
pub mod writer;

use color_code::ColorCode;
use writer::Writer;

pub static mut WRITER: Writer = Writer::new(ColorCode::default());
pub static SCREEN_WIDTH: usize = 80;
pub static SCREEN_HEIGHT: usize = 25;

#[macro_export]
macro_rules! print {
    // Case 1: Format + args*
    ($fmt:expr $(, $arg:tt)*) => {{
        use core::fmt::Write;
        unsafe {
            write!(WRITER, $fmt, $($arg)*).unwrap();
            WRITER.color = ColorCode::default();
        }
    }};

    // Case 2: Format + args* + color
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        use core::fmt::Write;
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
