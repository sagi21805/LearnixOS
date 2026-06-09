extern crate alloc;

use alloc::boxed::Box;

use common::{constants::REGULAR_PAGE_SIZE, ring_buffer::RingBuffer};

use crate::{
    color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

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

impl<const W: usize, const H: usize> GenericWriter
    for AdvancedWriter<W, H>
{
    fn backspace(&mut self) {
        unsafe {
            self.buffer.advance_write(-1);
        }
        self.buffer
            .write(ScreenChar::new(b' ', ColorCode::default()));
    }

    fn new_line(&mut self) {
        unsafe {
            self.buffer
                .advance_write((W - self.buffer.write_idx() % W) as isize);
        }
    }

    fn screen_height(&self) -> usize {
        H
    }

    fn screen_width(&self) -> usize {
        W
    }

    fn scroll_down(&mut self, lines: usize) {
        if lines == 0 {
            return;
        }
        unsafe {
            self.buffer.advance_write(
                ((W - self.buffer.write_idx() % W) + (W * (lines - 1)))
                    as isize,
            );
        }
        unsafe {
            // Read pointer is always on the start of the line.
            // So modulo is not needed.
            self.buffer.advance_read((W * lines) as isize);
        }
    }

    fn scroll_up(&mut self, lines: usize) {
        if lines == 0 {
            return;
        }
        unsafe {
            self.buffer.advance_write(
                (-1 * self.buffer.write_idx() as isize)
                    + (-1 * W as isize * (lines - 1) as isize),
            );
        }
        unsafe {
            // Read pointer is always on the start of the line.
            // So line offset is not needed.
            self.buffer.advance_read(-1 * W as isize * lines as isize);
        }
    }

    fn update(&self) {
        let vga = unsafe {
            ::core::slice::from_raw_parts_mut(
                0xb8000 as *mut ScreenChar,
                80 * 25,
            )
        };

        unsafe { self.buffer.read_bulk_no_advance(vga) };
    }

    fn write_cursor_position(&self) -> usize {
        self.buffer.write_idx()
    }

    fn write_vga_char(&mut self, char: ScreenChar) {
        self.buffer.write(char);
    }

    fn set_color(&mut self, color: Option<ColorCode>) {
        self.color = color.unwrap_or_default();
    }

    fn color(&self) -> Option<ColorCode> {
        Some(self.color)
    }
}
