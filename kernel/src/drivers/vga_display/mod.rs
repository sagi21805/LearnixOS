#![allow(unsafe_op_in_unsafe_fn)]

pub mod color_code;
mod screen_char;
mod writer;

use color_code::ColorCode;
use writer::Writer;

#[allow(private_interfaces)]
pub static mut WRITER: Writer = Writer::new();
static SCREEN_WIDTH: usize = 80;
static SCREEN_HEIGHT: usize = 25;

/// Prints formatted text to the VGA display without a newline.
///
/// # Examples
/// ```
/// print!("Hello, {}!", "world");
/// print!("Error: {}", error; color = ColorCode::new(Color::Red, Color::Black));
/// ```
///
/// # Parameters
/// - `$fmt`: The format string.
/// - `$arg`: Optional arguments to interpolate into the format string.
/// - `color = $color`: Optional named parameter to change the VGA text color for this print.
#[macro_export]
macro_rules! print {
    // Case 1: Standard print with optional arguments.
    ($fmt:expr $(, $arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::drivers::vga_display::WRITER;
        use $crate::drivers::vga_display::color_code::ColorCode;

        unsafe {
            #[allow(static_mut_refs)]
            write!(WRITER, $fmt $(, $arg)*).unwrap();
            WRITER.color = ColorCode::default();
        }
    }};

    // Case 2: Print with custom color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        use core::fmt::Write;
        use $crate::drivers::vga_display::WRITER;
        use $crate::drivers::vga_display::color_code::ColorCode;

        unsafe {
            WRITER.color = $color;
            write!(WRITER, $fmt, $($arg)*).unwrap();
            WRITER.color = ColorCode::default();
        }
    }};
}

/// Prints formatted text followed by a newline to the VGA display.
/// Same as the [`print!`] macro just with a `\n` attached to the end
///
/// # Examples
/// ```
/// println!("This will be printed to the screen!");
/// println!("User: {}, ID: {}", user, id; color = ColorCode::new(Color::Blue, Color::Black));
/// ```
#[macro_export]
macro_rules! println {
    // Case 1: Standard println with optional arguments.
    ($fmt:expr $(, $arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)*)
    };
    // Case 2: println with custom color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {
        $crate::print!(concat!($fmt, "\n") $(, $arg)* ; color = $color)
    };
}

/// Prints a standardized failure message in red color with optional formatting and message color.
///
/// # Examples
/// ```
/// fail_msg!("Failed to load module '{}'", module_name);
/// fail_msg!("Crash at address {:x}", addr; color = my_color);
/// ```
#[macro_export]
macro_rules! fail_msg {
    // Case 1: Print "FAIL" with formatted message.
    ($fmt:expr $(, $arg:tt)*) => {{
        $crate::print!("[");
        $crate::print!("FAIL" ; color = ColorCode::new(Color::Red, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    // Case 2: Print "FAIL" with custom message color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        $crate::print!("[");
        $crate::print!("FAIL" ; color = ColorCode::new(Color::Red, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }};
}

/// Prints a standardized success message in green color with optional formatting and message color.
///
/// # Examples
/// ```
/// ok_msg!("Successfully initialized '{}'", service_name);
/// ok_msg!("Everything looks good!" ; color = ColorCode);
/// ```
#[macro_export]
macro_rules! ok_msg {
    // Case 1: Print "OK" with formatted message.
    ($fmt:expr $(, $arg:tt)*) => {{
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new(Color::Green, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)*);
    }};

    // Case 2: Print "OK" with custom message color.
    ($fmt:expr $(, $arg:tt)* ; color = $color:expr) => {{
        $crate::print!("[");
        $crate::print!(" OK " ; color = ColorCode::new(Color::Green, Color::Black));
        $crate::print!("]: ");
        $crate::println!($fmt $(, $arg)* ; color = $color);
    }};
}

/// Clears the VGA screen using the current writer instance.
///
/// # Example
/// ```
/// clear!();
/// ```
#[macro_export]
macro_rules! clear {
    () => {{
        use core::fmt::Write;
        use $crate::screen::WRITER;
        use $crate::screen::color_code::ColorCode;

        WRITER.clear()
    }};
}
