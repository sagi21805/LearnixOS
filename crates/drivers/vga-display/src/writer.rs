extern crate alloc;

use core::ascii::Char;

use crate::{
    SCREEN_LOCK, color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

/// Writer implementation for the VGA driver.
pub struct SimpleWriter<const W: usize, const H: usize> {
    pub cursor_position: usize,
    pub color: ColorCode,
}

#[rustfmt::skip]
impl<const W: usize, const H: usize> const Default for SimpleWriter<W, H> {
    fn default() -> Self {
        Self {
            cursor_position: 0,
            color: ColorCode::default(),
        }
    }
}

impl<const W: usize, const H: usize> GenericWriter for SimpleWriter<W, H> {
    fn scroll_down(&mut self, _lines: usize) {
        unimplemented!()
    }

    fn scroll_up(&mut self, _lines: usize) {
        unimplemented!()
    }

    fn new_line(&mut self) {
        self.cursor_position =
            self.cursor_position + (W - (self.cursor_position % W))
    }

    fn backspace(&mut self) {
        self.cursor_position = self.cursor_position - 1;
        SCREEN_LOCK.lock()[self.cursor_position] = ScreenChar::default();
    }

    fn color(&self) -> ColorCode {
        self.color
    }

    fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

    fn screen_height(&self) -> usize {
        H
    }
    fn screen_width(&self) -> usize {
        W
    }

    fn update(&mut self) {}

    fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position;
    }
    fn write_cursor_position(&self) -> usize {
        self.cursor_position
    }
    fn write_vga_char(&mut self, char: ScreenChar) {
        SCREEN_LOCK.lock()[self.cursor_position] = char;
        self.cursor_position = self.cursor_position + 1;
    }
    fn write_char(&mut self, char: char) {
        let ascii =
            char.as_ascii().expect("Entered invalid ascii character");

        match ascii {
            Char::LineFeed => {
                self.new_line();
            }
            Char::Delete | Char::Backspace => {
                self.backspace();
            }
            _ => {
                if !ascii.is_control() {
                    self.write_vga_char(ScreenChar {
                        char: ascii,
                        color_code: self.color,
                    });
                }
            }
        }
        self.change_cursor_position_on_screen();
    }
}
