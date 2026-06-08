extern crate alloc;

use core::ascii::Char;

use alloc::boxed::Box;

use common::{
    constants::REGULAR_PAGE_SIZE,
    enums::{Port, VgaCommand},
    ring_buffer::RingBuffer,
};
use x86::instructions::port::PortExt;

use crate::{color_code::ColorCode, screen_char::ScreenChar};

pub struct AdvancedWriter<const W: usize, const H: usize> {
    pub color: ColorCode,
    pub buffer: RingBuffer<ScreenChar>,
}

impl<const W: usize, const H: usize> Default for AdvancedWriter<W, H> {
    fn default() -> Self {
        const BUFFER_SIZE: usize =
            REGULAR_PAGE_SIZE / size_of::<ScreenChar>();

        Self {
            color: ColorCode::default(),
            buffer: unsafe {
                RingBuffer::new(
                    Box::<[ScreenChar; BUFFER_SIZE]>::new_zeroed()
                        .assume_init(),
                )
            },
        }
    }
}

impl<const W: usize, const H: usize> AdvancedWriter<W, H> {
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
                    self.buffer.write(ScreenChar::new(char, self.color));
                }
            }
        }
        if self.cursor_position == (W * H) {
            self.scroll_down(1);
        }

        self.change_cursor_position_on_screen();
    }

    fn cursor_position(&self) -> usize {
        self.buffer.read_idx()
    }

    /// Scroll `lines` down.
    fn scroll_down(&mut self, lines: usize) {
        self.buffer.forward_advance_read(lines * W);
    }

    fn new_line(&mut self) {
        let offset = W - (self.buffer.read_idx() % W);
        self.buffer.forward_advance_read(offset);
    }

    fn backspace(&mut self) {
        unsafe {
            self.buffer.advance_write(-1);
        }
        self.buffer.write(ScreenChar::default());
    }

    fn change_cursor_position_on_screen(&self) {
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.buffer.read_idx() & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData
                .outb(((self.buffer.read_idx() >> 8) & 0xff) as u8);
        }
    }

    /// Clears the screen by setting all of the buffer bytes
    /// to zero
    fn clear(&mut self) {
        self.buffer.advance_to_write();
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
