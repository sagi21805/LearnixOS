extern crate alloc;

use core::{ascii::Char, cell::Cell};

use common::constants::VGA_BUFFER_PTR;

use crate::{
    color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

/// Writer implementation for the VGA driver.
pub struct SimpleWriter<const W: usize, const H: usize> {
    pub cursor_position: Cell<usize>,
    pub color: ColorCode,
    pub screen: &'static mut [ScreenChar],
}

#[rustfmt::skip]
impl<const W: usize, const H: usize> const Default for SimpleWriter<W, H> {
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
    fn screen_width(&self) -> usize {
        W
    }
    fn set_color(&mut self, color: Option<ColorCode>) {
        self.color = color.unwrap_or_default();
    }
    fn update(&mut self) {}
    fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position.set(position);
    }
    fn write_cursor_position(&self) -> usize {
        self.cursor_position.get()
    }
    fn write_vga_char(&mut self, char: ScreenChar) {
        self.screen[self.cursor_position.get()] = char;
        self.cursor_position.set(self.cursor_position.get() + 1);
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
