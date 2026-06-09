use core::ascii::Char;

use common::enums::{Port, VgaCommand};
use x86::instructions::port::PortExt;

use crate::{color_code::ColorCode, screen_char::ScreenChar};

pub struct Writer<'a> {
    pub inner: &'a mut dyn GenericWriter,
}

impl<'a> Writer<'a> {
    pub const fn new(inner: &'a mut dyn GenericWriter) -> Self {
        Self { inner }
    }

    pub const fn set_writer(&mut self, inner: &'a mut dyn GenericWriter) {
        self.inner = inner;
    }
}

pub trait GenericWriter {
    /// Update text to the VGA buffer
    fn update(&self);

    /// Write single char
    fn write_vga_char(&mut self, char: ScreenChar);

    /// Get cursor position
    fn write_cursor_position(&self) -> usize;

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
    fn color(&self) -> Option<ColorCode> {
        None
    }

    fn set_color(&mut self, color: Option<ColorCode>);

    /// Writes the given `char` to the screen with the color
    /// stored in self
    ///
    /// # Parameters
    ///
    /// - `char`: The char that will be printed to the screen
    fn write_char(&mut self, char: char) {
        let c = char.as_ascii().expect("Entered invalid ascii character");

        match c {
            Char::LineFeed => {
                self.new_line();
            }
            Char::Backspace | Char::Delete => {
                self.backspace();
            }
            _ => {
                if !c.is_control() {
                    self.write_vga_char(ScreenChar::new(
                        c.to_u8(),
                        self.color().unwrap_or_default(),
                    ));
                }
            }
        }
        if self.write_cursor_position()
            == (self.screen_width() * self.screen_height())
        {
            self.scroll_down(1);
        }

        self.change_cursor_position_on_screen();
    }
}

impl<'a> ::core::fmt::Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for c in s.chars() {
            self.inner.write_char(c);
        }
        Ok(())
    }
}
