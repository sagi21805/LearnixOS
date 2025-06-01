use super::color_code::ColorCode;
use super::screen_char::ScreenChar;
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};
use common::constants::addresses::VGA_BUFFER_PTR;

/// Writer implementation for the VGA driver.
///
/// This implementation will help track the wanted position to write to the screen
pub struct Writer {
    col: usize,
    row: usize,
    pub color: ColorCode,
}

impl Copy for ColorCode {}

impl Writer {
    /// Creates a new writer with the following parameters
    /// ```rust
    /// Self {
    ///     col: 0,
    ///     row: 0,
    ///     color: ColorCode::default(),
    /// }
    /// ```
    pub const fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            color: ColorCode::default(),
        }
    }

    /// Writes the given `char` to the screen with the color stored in self
    ///
    /// # Parameters
    ///
    /// - `char`: The char that will be printed to the screen
    fn write_char(&mut self, char: u8) {
        unsafe {
            match char {
                b'\n' => {
                    self.row += 1;
                    self.col = 0;
                }
                _ => {
                    (VGA_BUFFER_PTR as *mut ScreenChar)
                        .add((self.col + self.row * SCREEN_WIDTH) as usize)
                        .write_volatile(ScreenChar::new(char, self.color));
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
    }

    /// Clears the screen by setting all of the buffer bytes to zero
    pub fn clear(&mut self) {
        unsafe {
            (VGA_BUFFER_PTR as *mut ScreenChar)
                .write_bytes(b'\0', (SCREEN_WIDTH * SCREEN_HEIGHT) as usize);
            self.row = 0;
            self.col = 0;
        }
    }
}

impl core::fmt::Write for Writer {
    /// Print the given string to the string with the color in self
    ///
    /// # Parameters
    ///
    /// - `str`: The string that will be printed to the screen with the color in self
    ///
    /// # Safety
    /// THIS FUNCTION IS NOT THREAD SAFE AND NOT MARKED UNSAFE BECAUSE OF TRAIT IMPLEMENTATION!
    /// THE FUNCTION WILL ADD LOCK AND WILL BE SAFE IN THE FUTURE
    ///
    /// TODO: use lock in the future
    fn write_str(&mut self, str: &str) -> core::fmt::Result {
        for char in str.bytes() {
            self.write_char(char);
        }
        Ok(())
    }
}
