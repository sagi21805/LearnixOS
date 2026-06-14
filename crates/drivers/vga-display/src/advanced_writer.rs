extern crate alloc;

use alloc::boxed::Box;

use common::constants::{REGULAR_PAGE_SIZE, VGA_BUFFER_PTR};
use core::{ascii::Char, cell::Cell};

use crate::{
    color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

#[derive(Debug)]
pub struct AdvancedWriter<const W: usize, const H: usize> {
    pub color: ColorCode,
    pub cursor: Cell<usize>,
    pub screen_position: Cell<usize>,
    pub read_position: Cell<usize>,
    pub buffer: Box<[ScreenChar]>,
    pub backing: &'static mut [ScreenChar],
    pub row_table: Cell<[usize; H]>,
    pub line: Cell<usize>,
}

impl<const W: usize, const H: usize> Default for AdvancedWriter<W, H> {
    fn default() -> Self {
        const BUFFER_SIZE: usize =
            (10 * REGULAR_PAGE_SIZE) / size_of::<ScreenChar>();

        let vga = unsafe {
            ::core::slice::from_raw_parts_mut(
                VGA_BUFFER_PTR as *mut ScreenChar,
                80 * 25,
            )
        };

        Self {
            color: ColorCode::default(),
            buffer: unsafe {
                Box::<[ScreenChar; BUFFER_SIZE]>::new_zeroed()
                    .assume_init()
            },
            screen_position: Cell::new(0),
            read_position: Cell::new(0),
            cursor: Cell::new(0),
            backing: vga,
            row_table: Cell::new([0; H]),
            line: Cell::new(0),
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

    fn scroll_down(&self, lines: usize) {
        if self.screen_position.get() < W * H {
            return;
        }

        self.screen_position.set(
            self.screen_position.get().saturating_sub((H - lines) * W),
        );
        self.read_position
            .set(self.read_position.get().saturating_sub((H - lines) * W));
    }

    fn scroll_up(&self, lines: usize) {
        // if self.screen_start.get() > 0 {
        //     return;
        // }
        // self.screen_start
        //     .set(self.screen_start.get().saturating_sub(lines * W));
        // self.screen_position.set(self.screen_start.get());
    }

    fn update(&mut self) {
        if self.screen_position.get() >= W * H {
            self.scroll_down(1);
            return;
        }
        for char in &self.buffer.as_ref()
            [self.read_position.get()..self.cursor.get()]
        {
            match char.char {
                Char::Backspace | Char::Delete => {
                    self.screen_position
                        .set(self.screen_position.get().saturating_sub(1));

                    self.backing[self.screen_position.get()] =
                        ScreenChar::default();
                }
                Char::LineFeed => {
                    self.screen_position.set(
                        self.screen_position.get()
                            + (W - (self.screen_position.get() % W)),
                    );
                    self.line.set(self.line.get().saturating_add(1));

                    if self.screen_position.get() >= W * H {
                        self.scroll_down(1);
                        self.read_position.set(
                            self.read_position.get().saturating_add(1),
                        );
                        return;
                    }
                }
                _ => {
                    if self.screen_position.get() >= W * H {
                        self.scroll_down(1);
                        return;
                    }
                    self.backing[self.screen_position.get()] = *char;

                    self.screen_position
                        .set(self.screen_position.get().saturating_add(1));
                    self.read_position
                        .set(self.read_position.get().saturating_add(1));
                }
            }

            self.change_cursor_position_on_screen();
        }
    }

    fn write_cursor_position(&self) -> usize {
        self.screen_position.get()
    }

    fn set_cursor_position(&mut self, position: usize) {
        self.screen_position.set(position);
        self.cursor.set(position);
        // Copy to the buffer information from the backing.
        if self.read_position.get() != self.screen_position.get() {
            self.buffer
                [self.read_position.get()..self.screen_position.get()]
                .copy_from_slice(
                    &self.backing[self.read_position.get()
                        ..self.screen_position.get()],
                );
        }
    }

    fn write_vga_char(&mut self, char: ScreenChar) {
        self.buffer.as_mut()[self.cursor.get()] = char;
        self.cursor.set(self.cursor.get() + 1);
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
