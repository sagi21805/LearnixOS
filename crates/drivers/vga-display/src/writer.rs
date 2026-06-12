extern crate alloc;

use core::{ascii::Char, cell::Cell};

use crate::{
    color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};
use x86::instructions::port::PortExt;

/// Writer implementation for the VGA driver.
pub struct SimpleWriter<const W: usize, const H: usize> {
    pub cursor_position: Cell<usize>,
    pub color: ColorCode,
    pub screen: &'static mut [ScreenChar],
}

#[rustfmt::skip]
impl<const W: usize, const H: usize> const Default for Writer<W, H> {
    fn default() -> Self {
        Self {
            cursor_position: Cell::new(0),
            color: ColorCode::default(),
            screen: unsafe {
                core::slice::from_raw_parts_mut(
                    VGA_BUFFER_PTR as *mut ScreenChar,
                    W * H,
                )
            },
        }
    }
}

impl<const W: usize, const H: usize> GenericWriter for SimpleWriter<W, H> {
    fn scroll_down(&self, _lines: usize) {
        unimplemented!()
    }

    fn scroll_up(&self, _lines: usize) {
        unimplemented!()
    }

    fn new_line(&mut self) {
        self.cursor_position.set(
            self.cursor_position.get()
                + (W - (self.cursor_position.get() % W)),
        );
    }

    fn backspace(&mut self) {
        self.cursor_position.set(self.cursor_position.get() - 1);
        self.screen[self.cursor_position.get()] = ScreenChar::default();
    }

    fn color(&self) -> Option<ColorCode> {
        Some(self.color)
    }

    fn screen_height(&self) -> usize {
        H
    }
}

impl<const W: usize, const H: usize> core::fmt::Write for Writer<W, H> {
    /// Print the given string to the string with the color
    /// in self
    ///
    /// # Parameters
    ///
    /// - `str`: The string that will be printed to the screen with the
    ///   color in self
    ///
    /// # Safety
    /// THIS FUNCTION IS NOT THREAD SAFE AND NOT MARKED
    /// UNSAFE BECAUSE OF TRAIT IMPLEMENTATION!
    /// THE FUNCTION WILL ADD LOCK AND WILL BE SAFE IN THE
    /// FUTURE
    ///
    /// TODO: use lock in the future
    fn write_str(&mut self, str: &str) -> core::fmt::Result {
        for char in str.bytes() {
            self.write_char(char);
        }
        Ok(())
    }

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
                        c,
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
