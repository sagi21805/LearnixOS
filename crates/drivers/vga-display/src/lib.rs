#![no_std]
#![feature(ascii_char)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(ascii_char_variants)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
pub mod advanced_writer;
pub mod ansi;
pub mod color_code;
pub mod generic_writer;
pub mod screen_char;
pub mod writer;

use color_code::ColorCode;
use common::{
    constants::VGA_BUFFER_PTR,
    enums::{Port, VgaCommand},
    volatile::Volatile,
};
use x86::instructions::port::PortExt;

use core::{
    ascii::Char,
    fmt::{self, Write},
    ptr::with_exposed_provenance_mut,
};

use crate::{
    advanced_writer::AdvancedWriter, generic_writer::Writer,
    screen_char::ScreenChar, writer::SimpleWriter,
};

use sync::{mutex::SpinMutex, once::Once};

/// Columns of the VGA text mode.
pub const WIDTH: usize = 80;
/// Rows of the VGA text mode.
pub const HEIGHT: usize = 25;
const CELLS: usize = WIDTH * HEIGHT;

pub static SCREEN: SpinMutex<Screen> = SpinMutex::new(Screen::new());

/// The writer that needs no heap, used by [`WRITER`] until a better one is
/// set, and by the panic handler.
// SAFETY: only `WRITER`'s `leak` and the panic handler's `force_unlock`
// reach it.
pub static SIMPLE_WRITER: SpinMutex<SimpleWriter<80, 25>> =
    unsafe { SpinMutex::new_locked(SimpleWriter::default()) };

/// The writer behind [`print!`] and [`println!`].
// SAFETY: this is the only `leak` of `SIMPLE_WRITER`.
pub static WRITER: SpinMutex<Writer<'static>> =
    SpinMutex::new(Writer::new(unsafe { SIMPLE_WRITER.leak() }));

/// The writer with a scrollback buffer, which replaces [`SIMPLE_WRITER`]
/// in [`WRITER`] once the heap works.
pub static ADVANCED_WRITER: Once<SpinMutex<AdvancedWriter<80, 25>>> =
    Once::uninit();

pub struct Screen {
    screen_position: usize,
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
    const fn new() -> Self { Self { screen_position: 0 } }

    /// VGA text memory, which `&mut self` keeps exclusive because
    /// [`SCREEN`] is the only `Screen`.
    fn vga_cells(&mut self) -> &mut [Volatile<ScreenChar>; CELLS] {
        let cells = with_exposed_provenance_mut(VGA_BUFFER_PTR as usize);
        // SAFETY: the boot page tables map VGA text memory, and any byte
        // is a valid `ScreenChar`.
        unsafe { &mut *cells }
    }

    pub fn write_char(&mut self, c: ScreenChar) -> WriteInfo {
        let info = match Char::from_u8(c.glyph) {
            Some(Char::LineFeed) => self.new_line(),
            Some(Char::Backspace | Char::Delete) => self.backspace(),
            _ => {
                if self.screen_position < CELLS {
                    let position = self.screen_position;
                    self.vga_cells()[position].write(c);
                    self.screen_position += 1;
                    if self.screen_position.is_multiple_of(WIDTH) {
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

        info
    }

    pub fn new_line(&mut self) -> WriteInfo {
        let offset = self.screen_position % WIDTH;
        let new_position = self.screen_position + (WIDTH - offset);

        let position = self.screen_position;
        let line_end = new_position.min(CELLS);
        self.vga_cells()[position..line_end]
            .iter_mut()
            .for_each(|cell| cell.write(ScreenChar::default()));

        if new_position >= CELLS {
            self.screen_position = CELLS - 1;
            WriteInfo::EndOfScreen
        } else {
            self.screen_position = new_position;
            WriteInfo::LineUp
        }
    }

    pub fn backspace(&mut self) -> WriteInfo {
        if self.screen_position > 0 {
            self.screen_position -= 1;
            let position = self.screen_position;
            self.vga_cells()[position].write(ScreenChar::default());
            // Backspace to the first character of the line
            if self.screen_position % WIDTH == WIDTH - 1 {
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
        self.vga_cells()[CELLS - lines * WIDTH..]
            .iter_mut()
            .for_each(|cell| cell.write(ScreenChar::default()));
    }

    fn scroll_down(&mut self, lines: usize) {
        self.vga_cells()[..lines * WIDTH]
            .iter_mut()
            .for_each(|cell| cell.write(ScreenChar::default()));
    }

    pub fn reset_cursor(&mut self) { self.screen_position = 0; }

    pub fn clear(&mut self) {
        self.vga_cells()
            .iter_mut()
            .for_each(|cell| cell.write(ScreenChar::default()));
    }
}

pub fn vga_print(args: fmt::Arguments<'_>, color: Option<ColorCode>) {
    let writer = &mut WRITER.lock().inner;

    writer.set_color(color.unwrap_or_default());

    writer.write_fmt(args).unwrap();

    writer.set_color(ColorCode::default());
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
