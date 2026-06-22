extern crate alloc;

use core::{
    ascii::Char,
    fmt::{Arguments, Write},
};

use common::enums::Color;
use sync::mutex::SpinMutex;

use crate::{
    SCREEN, Screen, color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

/// Writer implementation for the VGA driver.
pub struct SimpleWriter<const W: usize, const H: usize> {
    pub color: ColorCode,
    screen: &'static SpinMutex<Screen>,
}

impl<const W: usize, const H: usize> SimpleWriter<W, H> {
    pub fn panic_message(&mut self, message: Arguments<'_>) {
        let red = ColorCode::new()
            .foreground(Color::Red)
            .background(Color::Black);

        let yellow = ColorCode::new()
            .foreground(Color::Yellow)
            .background(Color::Black);
        self.color = yellow;
        GenericWriter::write_char(self, '\n');
        GenericWriter::write_char(self, '[');
        self.color = red;
        self.write_str(" FAIL ").unwrap();
        self.color = yellow;
        self.write_str("]: ").unwrap();
        self.write_fmt(message).unwrap();
    }
}

#[rustfmt::skip]
impl<const W: usize, const H: usize> const Default for SimpleWriter<W, H> {
    fn default() -> Self {
        Self {
            color: ColorCode::default(),
            screen: &SCREEN,
        }
    }
}

impl<const W: usize, const H: usize> GenericWriter for SimpleWriter<W, H> {
    fn scroll_down(&mut self, lines: usize) {
        self.screen.lock().scroll_down(lines);
    }

    fn set_cursor_position(&mut self, _p: usize) {}

    fn write_cursor_position(&self) -> usize {
        self.screen.lock().screen_position
    }

    fn scroll_up(&mut self, lines: usize) {
        self.screen.lock().scroll_up(lines);
    }

    fn new_line(&mut self) { self.screen.lock().new_line(); }

    fn backspace(&mut self) { self.screen.lock().backspace(); }

    fn color(&self) -> ColorCode { self.color }

    fn set_color(&mut self, color: ColorCode) { self.color = color; }

    fn screen_height(&self) -> usize { H }

    fn screen_width(&self) -> usize { W }

    fn update(&mut self) {}

    fn write_vga_char(&mut self, char: ScreenChar) {
        self.screen.lock().write_char(char);
    }

    fn write_char(&mut self, char: char) {
        let ascii =
            char.as_ascii().expect("Entered invalid ascii character");

        match ascii {
            Char::Delete | Char::Backspace => {
                self.backspace();
            }
            _ => {
                if !ascii.is_control() || ascii == Char::LineFeed {
                    self.write_vga_char(ScreenChar {
                        char: ascii,
                        color_code: self.color,
                    });
                }
            }
        }
    }
}

impl<const W: usize, const H: usize> ::core::fmt::Write
    for SimpleWriter<W, H>
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            GenericWriter::write_char(self, c);
        }
        Ok(())
    }
}
