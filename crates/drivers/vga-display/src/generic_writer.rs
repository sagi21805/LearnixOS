use common::enums::{Port, VgaCommand};

use sync::mutex::SpinMutex;
#[cfg(not(feature = "host"))]
use x86::instructions::port::PortExt;

use crate::{color_code::ColorCode, screen_char::ScreenChar};

pub struct Writer<'a> {
    pub inner: SpinMutex<&'a mut dyn GenericWriter>,
}

impl<'a> Writer<'a> {
    pub const fn new(inner: &'a mut dyn GenericWriter) -> Self {
        Self {
            inner: SpinMutex::new(inner),
        }
    }

    pub fn set_writer(&mut self, inner: &'a mut dyn GenericWriter) {
        let cursor = self.inner.lock().write_cursor_position();
        *self.inner.lock() = inner;
        self.inner.lock().set_cursor_position(cursor);
    }
}

pub trait GenericWriter: Send {
    /// Update text to the VGA buffer
    fn update(&mut self);

    /// Write single char
    fn write_vga_char(&mut self, char: ScreenChar);

    /// Get cursor position
    fn write_cursor_position(&self) -> usize;

    fn set_cursor_position(&mut self, position: usize);

    // Go down a line
    fn new_line(&mut self);

    /// Delete last character
    fn backspace(&mut self);

    /// Get screen width
    fn screen_width(&self) -> usize;

    /// Get screen height
    fn screen_height(&self) -> usize;

    /// Scroll down by `lines`
    fn scroll_down(&mut self, lines: usize);

    /// Scroll up by `lines`
    fn scroll_up(&mut self, lines: usize);

    /// Change cursor position on screen
    fn change_cursor_position_on_screen(&self) {
        #[cfg(feature = "host")]
        // Disable this function when running tests on the host computer
        return;

        #[cfg(not(feature = "host"))]
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData
                .outb((self.write_cursor_position() & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData
                .outb(((self.write_cursor_position() >> 8) & 0xff) as u8);
        }
    }

    // Return the color for the next printed character
    fn color(&self) -> ColorCode;

    fn set_color(&mut self, color: ColorCode);

    /// Writes the given `char` to the screen with the color
    /// stored in self
    ///
    /// # Parameters
    ///
    /// - `char`: The char that will be printed to the screen
    fn write_char(&mut self, char: char);
}

impl core::fmt::Write for dyn GenericWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
