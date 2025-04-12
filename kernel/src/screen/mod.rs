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
        use $crate::screen::color_code::ColorCode;

        unsafe {
            #[allow(static_mut_refs)]
            write!(WRITER, $fmt $(,$arg)*).unwrap();
            WRITER.color = ColorCode::default();
        }
    }};

    // Case 2: Format + args* + color
    ($fmt:expr $(,$arg:tt)* ; color = $color:expr) => {{
        use core::fmt::Write;
        use $crate::screen::WRITER;
        use $crate::screen::color_code::ColorCode;
        unsafe {
            WRITER.color = $color;
            write!(WRITER, $fmt, $($arg)*).unwrap();
            WRITER.color = ColorCode::default();
        }
    }};
}

#[macro_export]
macro_rules! println {

    // Case 2: Format + args*
    ($fmt:expr $(, $arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)*)
    };
    // Case 1: Format + args* + color
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)* ; color = $color)
    };
}

#[macro_export]
macro_rules! println_fail {
    ($fmt:expr $(, $arg:tt)*) => {{
        $crate::print!("[");
        $crate::print!(" FAIL " ; color = ColorCode::new(Color::Red, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    ($fmt:expr $(,$arg:tt)* ; color = $color:expr) => {
        $crate::print!("[");
        $crate::print!(" FAIL " ; color = ColorCode::new(Color::Red, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }
}

#[macro_export]
macro_rules! println_ok {
    ($fmt:expr $(, $arg:tt)*) => {{
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new(Color::Green, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    ($fmt:expr $(,$arg:tt)* ; color = $color:expr) => {
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new(Color::Green, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }
}

#[macro_export]
macro_rules! clear {
    () => {{
        use core::fmt::Write;
        use $crate::screen::WRITER;
        use $crate::screen::color_code::ColorCode;
        WRITER.clear()
    }};
}
