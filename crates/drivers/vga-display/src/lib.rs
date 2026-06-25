#![no_std]
#![feature(ascii_char)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(ascii_char_variants)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
pub mod advanced_writer;
pub mod color_code;
pub mod generic_writer;
pub mod screen_char;
pub mod writer;

use color_code::ColorCode;
use common::{
    constants::VGA_BUFFER_PTR,
    enums::{Port, VgaCommand},
    late_init::LateInit,
};
use x86::instructions::port::PortExt;

use core::{
    ascii::Char,
    fmt::{self, Write},
};

use crate::{generic_writer::Writer, screen_char::ScreenChar};

use sync::mutex::SpinMutex;

pub static SCREEN: LateInit<SpinMutex<Screen>> = LateInit::uninit();

pub struct Screen {
    buffer: &'static mut [ScreenChar],
    screen_position: usize,
    pub width: usize,
    pub height: usize,
}

pub enum WriteInfo {
    /// Reached the end of the screen.
    EndOfScreen,
    /// Write was on the same line.
    SameLine,
    /// Went one line up
    LineUp,
    /// Went one line down
    LineDown,
}

impl Screen {
    pub fn write_char(&mut self, c: ScreenChar) -> WriteInfo {
        let info = match c.char {
            Char::LineFeed => self.new_line(),
            Char::Backspace | Char::Delete => self.backspace(),
            _ => {
                if self.screen_position < self.buffer.len() {
                    let position = &mut self.buffer[self.screen_position];
                    unsafe {
                        ::core::ptr::write_volatile(position as *mut _, c);
                    }
                    self.screen_position += 1;
                    if self.screen_position % self.width == 0 {
                        WriteInfo::LineUp
                    } else {
                        WriteInfo::SameLine
                    }
                } else {
                    WriteInfo::EndOfScreen
                }
            }
        };
        self.change_cursor_position_on_screen();

        return info;
    }

    pub fn new_line(&mut self) -> WriteInfo {
        let offset = self.screen_position % self.width;
        for i in 0..self.width - offset {
            self.buffer[self.screen_position + i] = ScreenChar::default()
        }

        let new_position = self.screen_position + (self.width - offset);

        if new_position >= self.buffer.len() {
            self.screen_position = self.buffer.len() - 1;
            WriteInfo::EndOfScreen
        } else {
            self.screen_position = new_position;
            WriteInfo::LineUp
        }
    }

    pub fn backspace(&mut self) -> WriteInfo {
        if self.screen_position > 0 {
            self.screen_position -= 1;
            self.buffer[self.screen_position] = ScreenChar::default();
            // Backspace to the first character of the line
            if self.screen_position % self.width == self.width - 1 {
                return WriteInfo::LineDown;
            }
        }
        WriteInfo::SameLine
    }

    pub fn cursor(&self) -> usize { self.screen_position }

    /// Change cursor position on screen
    fn change_cursor_position_on_screen(&self) {
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.screen_position & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData.outb(((self.screen_position >> 8) & 0xff) as u8);
        }
    }

    fn scroll_up(&mut self, lines: usize) {
        let anchor = lines * self.width;
        let len = self.buffer.len();
        self.buffer[len - anchor..].fill(ScreenChar::default());
    }

    fn scroll_down(&mut self, lines: usize) {
        let anchor = lines * self.width;
        self.buffer[0..anchor].fill(ScreenChar::default());
    }

    pub fn reset_cursor(&mut self) { self.screen_position = 0; }

    pub fn clear(&mut self) { self.buffer.fill(ScreenChar::default()); }
}

unsafe extern "Rust" {
    static WRITER: SpinMutex<Writer<'static>>;
}

pub fn vga_init() {
    let screen = unsafe {
        core::slice::from_raw_parts_mut(
            VGA_BUFFER_PTR as *mut ScreenChar,
            80 * 25,
        )
    };

    SCREEN.init(SpinMutex::new(Screen {
        buffer: screen,
        screen_position: 0,
        width: 80,
        height: 25,
    }));
}

pub fn vga_print(args: fmt::Arguments<'_>, color: Option<ColorCode>) {
    unsafe {
        let writer = &mut WRITER.lock().inner;

        writer.set_color(color.unwrap_or_default());

        writer.write_fmt(args).unwrap();

        writer.set_color(ColorCode::default());
    }
}
#[unsafe(no_mangle)]
pub fn kprint(args: fmt::Arguments<'_>) { vga_print(args, None); }

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
