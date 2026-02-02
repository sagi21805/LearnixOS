use core::ascii::Char;
use core::ptr::NonNull;

use super::color_code::ColorCode;
use super::screen_char::ScreenChar;
use common::constants::addresses::VGA_BUFFER_PTR;
use common::enums::{Port, VgaCommand};
use cpu_utils::instructions::port::PortExt;

// ANCHOR: writer
/// Writer implementation for the VGA driver.
pub struct Writer<const W: usize, const H: usize> {
    pub cursor_position: usize,
    pub color: ColorCode,
    pub screen: NonNull<[ScreenChar]>,
}
// ANCHOR_END: writer

// ANCHOR: writer_default
impl<const W: usize, const H: usize> const Default for Writer<W, H> {
    fn default() -> Self {
        Self {
            cursor_position: 0,
            color: ColorCode::default(),
            screen: unsafe {
                core::slice::from_raw_parts_mut(
                    VGA_BUFFER_PTR as *mut ScreenChar,
                    W * H + 1,
                )
            },
        }
    }
}
// ANCHOR_END: writer_default

impl<const W: usize, const H: usize> Writer<W, H> {
    /// Writes the given `char` to the screen with the color
    /// stored in self
    ///
    /// # Parameters
    ///
    /// - `char`: The char that will be printed to the screen
    fn write_char(&mut self, char: u8) {
        // ANCHOR: handle_char
        let c =
            Char::from_u8(char).expect("Entered invalid ascii character");
        match c {
            Char::LineFeed => {
                self.new_line();
            }
            Char::Backspace | Char::Delete => {
                self.backspace();
            }
            _ => {
                if !c.is_control() {
                    self.screen[self.cursor_position] =
                        ScreenChar::new(char, self.color);
                    self.cursor_position += 1;
                }
            }
        }
        if self.cursor_position > W * H {
            self.scroll_down(1);
        }
        // ANCHOR_END: handle_char

        // ANCHOR: change_position
        self.change_cursor_position_on_screen();
        // ANCHOR_END: change_position
    }

    // ANCHOR: scroll_down
    /// Scroll `lines` down.
    fn scroll_down(&mut self, lines: usize) {
        let lines_index = W * (H - lines) + 1;

        // Copy the buffer to the left
        self.screen.copy_within(lines * W.., 0);

        // Fill remaining place with empty characters
        for x in &mut self.screen[lines_index..] {
            *x = ScreenChar::default()
        }

        self.cursor_position -= lines * W;
    }
    // ANCHOR_END: scroll_down

    // ANCHOR: new_line
    fn new_line(&mut self) {
        self.cursor_position += W - (self.cursor_position % W)
    }
    // ANCHOR_END: new_line

    // ANCHOR: backspace
    fn backspace(&mut self) {
        self.cursor_position -= 1;
        self.screen[self.cursor_position] = ScreenChar::default();
    }
    // ANCHOR_END: backspace

    // ANCHOR: change_position_on_screen
    fn change_cursor_position_on_screen(&self) {
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.cursor_position & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData.outb(((self.cursor_position >> 8) & 0xff) as u8);
        }
    }
    // ANCHOR_END: change_position_on_screen

    // ANCHOR: clear
    /// Clears the screen by setting all of the buffer bytes
    /// to zero
    fn clear(&mut self) {
        self.screen.fill(ScreenChar::default());
        self.cursor_position = 0;
    }
    // ANCHOR_END: clear
}

// ANCHOR: format_impl
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
}
// ANCHOR_END: format_impl
