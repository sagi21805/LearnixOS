extern crate alloc;

use alloc::boxed::Box;

use common::{constants::REGULAR_PAGE_SIZE, ring_buffer::RingBuffer};
use core::ascii::Char;

use crate::{
    color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

#[derive(Debug)]
pub struct AdvancedWriter<const W: usize, const H: usize> {
    pub color: ColorCode,
    pub screen_start: usize,
    pub cursor: usize,
    pub screen_position: usize,
    pub buffer: Box<[ScreenChar]>,
    pub backing: &'static mut [ScreenChar],
}

impl<const W: usize, const H: usize> Default for AdvancedWriter<W, H> {
    fn default() -> Self {
        const BUFFER_SIZE: usize =
            REGULAR_PAGE_SIZE / size_of::<ScreenChar>();

        let vga = unsafe {
            ::core::slice::from_raw_parts_mut(
                0xb8000 as *mut ScreenChar,
                80 * 25,
            )
        };

        Self {
            color: ColorCode::default(),
            buffer: unsafe {
                Box::<[ScreenChar; BUFFER_SIZE]>::new_zeroed()
                    .assume_init()
            },
            screen_start: 0,
            screen_position: 0,
            cursor: 0,
            backing: vga,
        }
    }
}

impl<const W: usize, const H: usize> GenericWriter
    for AdvancedWriter<W, H>
{
    fn backspace(&mut self) {}

    fn new_line(&mut self) {}

    fn screen_height(&self) -> usize {
        H
    }

    fn screen_width(&self) -> usize {
        W
    }

    fn scroll_down(&mut self, lines: usize) {
        // if lines == 0 {
        //     return;
        // }
        // unsafe {
        //     self.buffer.advance_write(
        //         ((W - self.buffer.write_idx() % W) + (W * (lines - 1)))
        //             as isize,
        //     );
        // }
        // unsafe {
        //     // Read pointer is always on the start of the line.
        //     // So modulo is not needed.
        //     self.buffer.advance_read((W * lines) as isize);
        // }
    }

    fn scroll_up(&mut self, lines: usize) {
        // if lines == 0 {
        //     return;
        // }
        // unsafe {
        //     self.buffer.advance_write(
        //         (-1 * self.buffer.write_idx() as isize)
        //             + (-1 * W as isize * (lines - 1) as isize),
        //     );
        // }
        // unsafe {
        //     // Read pointer is always on the start of the line.
        //     // So line offset is not needed.
        //     self.buffer.advance_read(-1 * W as isize * lines as isize);
        // }
    }

    fn update(&mut self) {
        for char in &self.buffer.as_ref()[self.screen_position
            ..self.cursor.min(self.screen_position + W * H)]
        {
            match char.char {
                Char::Backspace | Char::Delete => {
                    self.screen_position =
                        self.screen_position.saturating_sub(1);
                    self.backing[self.screen_position] =
                        ScreenChar::default();
                }
                Char::LineFeed => {
                    self.screen_position = self.screen_position + W
                        - (self.screen_position % W);
                }
                _ => {
                    self.backing[self.screen_position] = *char;
                    self.screen_position =
                        self.screen_position.saturating_add(1);
                }
            }
            #[cfg(not(test))]
            self.change_cursor_position_on_screen();
        }
    }

    fn write_cursor_position(&self) -> usize {
        self.screen_position
    }

    fn set_cursor_position(&mut self, position: usize) {
        self.screen_position = position;
        self.cursor = self.screen_position;
    }

    fn write_vga_char(&mut self, char: ScreenChar) {
        self.buffer.as_mut()[self.cursor] = char;
        self.cursor += 1;
    }

    fn set_color(&mut self, color: Option<ColorCode>) {
        self.color = color.unwrap_or_default();
    }

    fn color(&self) -> Option<ColorCode> {
        Some(self.color)
    }

    fn write_char(&mut self, char: char) {
        let c = char.as_ascii().expect("Entered invalid ascii character");
        self.write_vga_char(ScreenChar::new(
            c,
            self.color().unwrap_or_default(),
        ));
    }
}
