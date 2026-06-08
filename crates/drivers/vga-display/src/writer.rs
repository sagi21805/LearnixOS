extern crate alloc;

use alloc::boxed::Box;

use core::ascii::Char;

use common::{
    constants::VGA_BUFFER_PTR,
    enums::{Port, VgaCommand},
    ring_buffer::RingBuffer,
};
use x86::instructions::port::PortExt;

/// Writer implementation for the VGA driver.
pub struct Writer<const W: usize, const H: usize> {
    pub cursor_position: usize,
    pub color: ColorCode,
    pub screen: &'static mut [ScreenChar],
    pub offscreen: Option<RingBuffer<ScreenChar>>,
}

#[rustfmt::skip]
impl<const W: usize, const H: usize> const Default for Writer<W, H> {
    fn default() -> Self {
        Self {
            cursor_position: 0,
            color: ColorCode::default(),
            screen: unsafe {
                core::slice::from_raw_parts_mut(
                    VGA_BUFFER_PTR as *mut ScreenChar,
                    W * H,
                )
            },
            offscreen: None,
        }
    }
}

impl<const W: usize, const H: usize> Writer<W, H> {
    /// Writes the given `char` to the screen with the color
    /// stored in self
    ///
    /// # Parameters
    ///
    /// - `char`: The char that will be printed to the screen
    fn write_char(&mut self, char: u8) {
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
        if self.cursor_position == (W * H) {
            self.scroll_down(1);
        }

        self.change_cursor_position_on_screen();
    }

    /// Scroll `lines` down.
    pub fn scroll_down(&mut self, lines: usize) {
        let lines_index = W * (H - lines);
        let region_size = lines * W;

        // Copy to offscreen buffer
        if let Some(ref mut buf) = self.offscreen {
            for x in &mut self.screen[..region_size] {
                buf.write(*x);
            }
        }

        // Copy the buffer to the left
        self.screen.copy_within(region_size.., 0);

        // Fill remaining place with empty characters
        for x in &mut self.screen[lines_index..] {
            *x = self
                .offscreen
                .as_mut()
                .and_then(|buf| buf.read())
                .unwrap_or_default();
        }

        self.cursor_position =
            self.cursor_position.saturating_sub(lines * W);
    }

    /// Scroll `lines` up.
    pub fn scroll_up(&mut self, lines: usize) {
        let lines_index = W * (H - lines);
        let region_size = lines * W;

        // Copy to offscreen buffer
        if let Some(ref mut buf) = self.offscreen {
            for x in &mut self.screen[lines_index..] {
                buf.write(*x);
            }
        }

        // Copy the buffer to the left
        self.screen.copy_within(..lines_index, region_size);

        // Fill remaining place with empty characters
        for x in &mut self.screen[..region_size] {
            *x = self
                .offscreen
                .as_mut()
                .and_then(|buf| buf.read())
                .unwrap_or_default();
        }

        self.cursor_position =
            self.cursor_position.saturating_sub(lines * W);
    }

    fn new_line(&mut self) {
        self.cursor_position += W - (self.cursor_position % W)
    }

    fn backspace(&mut self) {
        self.cursor_position -= 1;
        self.screen[self.cursor_position] = ScreenChar::default();
    }

    fn change_cursor_position_on_screen(&self) {
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.cursor_position & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData.outb(((self.cursor_position >> 8) & 0xff) as u8);
        }
    }

    /// Clears the screen by setting all of the buffer bytes
    /// to zero
    fn clear(&mut self) {
        self.screen.fill(ScreenChar::default());
        self.cursor_position = 0;
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
}
