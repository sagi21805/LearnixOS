use core::ptr;

use crate::println;

use super::color_code::ColorCode;
use super::screen_char::ScreenChar;
use super::{SCREEN_HEIGHT, SCREEN_WIDTH};
use common::constants::addresses::VGA_BUFFER_PTR;
use common::enums::{Port, VgaCommand};
use cpu_utils::instructions::port::PortExt;

/// Writer implementation for the VGA driver.
///
/// This implementation will help track the wanted position to write to the screen
pub struct Writer {
    pub cursor_position: usize,
    pub color: ColorCode,
}

impl Copy for ColorCode {}

impl Writer {
    /// Creates a default writer
    pub const fn default() -> Self {
        Self {
            cursor_position: 0,
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
                    self.new_line();
                }
                b'\x08' => {
                    self.backspace();
                }
                32..128 => {
                    (VGA_BUFFER_PTR as *mut ScreenChar)
                        .add(self.cursor_position)
                        .write_volatile(ScreenChar::new(char, self.color));
                    self.cursor_position += 1;
                }
                _ => {}
            }

            if self.cursor_position >= (SCREEN_WIDTH * SCREEN_HEIGHT) {
                self.scroll_down(1);
            }
            self.change_cursor_position_on_screen();
        }
    }

    /// Scroll `lines` down.
    fn scroll_down(&mut self, lines: usize) {
        let lines_index = SCREEN_WIDTH * (SCREEN_HEIGHT - lines);
        unsafe {
            // Copy the buffer to the left
            ptr::copy(
                (VGA_BUFFER_PTR as *mut ScreenChar).add(SCREEN_WIDTH),
                VGA_BUFFER_PTR as *mut ScreenChar,
                lines_index,
            );
            // Fill remaining place with empty characters
            for i in 0..SCREEN_WIDTH {
                ptr::write_volatile(
                    (VGA_BUFFER_PTR as *mut ScreenChar).add(lines_index + i),
                    ScreenChar::default(),
                );
            }
        }
        self.cursor_position -= lines * SCREEN_WIDTH;
    }

    fn new_line(&mut self) {
        self.cursor_position += SCREEN_WIDTH - (self.cursor_position % SCREEN_WIDTH)
    }

    fn backspace(&mut self) {
        self.cursor_position -= 1;
        unsafe {
            (VGA_BUFFER_PTR as *mut ScreenChar)
                .add(self.cursor_position)
                .write_volatile(ScreenChar::new(b' ', self.color));
        }
    }

    fn change_cursor_position_on_screen(&self) {
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.cursor_position & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData.outb(((self.cursor_position >> 8) & 0xff) as u8);
        }
    }

    /// Clears the screen by setting all of the buffer bytes to zero
    fn clear(&mut self) {
        unsafe {
            for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
                ptr::write_volatile(
                    (VGA_BUFFER_PTR as *mut ScreenChar).add(i),
                    ScreenChar::default(),
                );
            }
            self.cursor_position = 0;
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
