use super::color_code::ColorCode;
use super::screen_char::ScreenChar;
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};
use constants::addresses::VGA_BUFFER_PTR;

pub struct Writer {
    col: usize,
    row: usize,
    pub color: ColorCode,
}

impl Writer {
    pub const fn new(color: ColorCode) -> Self {
        Self {
            color,
            col: 0,
            row: 0,
        }
    }

    fn write_char(&mut self, char: u8) {
        unsafe {
            (VGA_BUFFER_PTR as *mut ScreenChar)
                .add(self.col + self.row * SCREEN_WIDTH)
                .write_volatile(ScreenChar::new(char, self.color));
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            (VGA_BUFFER_PTR as *mut ScreenChar).write_bytes(b'\0', SCREEN_WIDTH * SCREEN_HEIGHT);
            self.row = 0;
            self.col = 0;
        }
    }
}

impl core::fmt::Write for Writer {
    /// Print the messege in the given color
    /// IMPORTANT: THIS FUNCTION IS NOT THREAD SAFE!
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for char in s.bytes() {
            match char {
                b'\n' => {
                    self.row += 1;
                    self.col = 0;
                }
                _ => {
                    self.write_char(char);
                    self.col += 1;
                }
            }
            if self.col >= SCREEN_WIDTH {
                self.col = 0;
                self.row += 1;
            }
            if self.row >= SCREEN_HEIGHT {
                self.col = 0;
                self.row = 0;
            }
        }
        Ok(())
    }
}
