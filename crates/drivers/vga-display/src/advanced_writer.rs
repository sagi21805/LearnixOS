extern crate alloc;

use alloc::boxed::Box;

use common::constants::{REGULAR_PAGE_SIZE, VGA_BUFFER_PTR};
use core::{ascii::Char, cell::Cell};

use crate::{
    color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

use sync::mutex::SpinMutex;

pub struct AdvancedWriter<const W: usize, const H: usize> {
    pub color: ColorCode,
    pub cursor: Cell<usize>,
    pub screen_position: Cell<usize>,
    pub read_position: Cell<usize>,
    pub buffer: SpinMutex<Box<[ScreenChar]>>,
    pub row_table: SpinMutex<Box<[u16]>>,
    pub line: Cell<usize>,
}

impl<const W: usize, const H: usize> Default for AdvancedWriter<W, H> {
    fn default() -> Self {
        const BUFFER_SIZE: usize =
            (2 * REGULAR_PAGE_SIZE) / size_of::<ScreenChar>();

        Self {
            color: ColorCode::default(),
            buffer: SpinMutex::new(unsafe {
                Box::<[ScreenChar; BUFFER_SIZE]>::new_zeroed()
                    .assume_init()
            }),
            screen_position: Cell::new(0),
            read_position: Cell::new(0),
            cursor: Cell::new(0),
            row_table: SpinMutex::new(unsafe {
                Box::<[u16; H]>::new_zeroed().assume_init()
            }),
            line: Cell::new(0),
        }
    }
}

impl<const W: usize, const H: usize> AdvancedWriter<W, H> {
    fn screen_line(&self) -> usize {
        self.screen_position.get() / W
    }
    fn screen_line_offset(&self) -> usize {
        self.screen_position.get() % W
    }
    fn cursor_line(&self) -> usize {
        self.cursor.get() / W
    }
    fn cursor_line_offset(&self) -> usize {
        self.cursor.get() % W
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
        if self.screen_position.get() < W * H {
            return;
        }

        self.screen_position.set(
            self.screen_position.get().saturating_sub((H - lines) * W),
        );
        self.read_position
            .set(self.read_position.get().saturating_sub((H - lines) * W));
    }

    fn scroll_up(&mut self, lines: usize) {
        // if self.screen_start.get() > 0 {
        //     return;
        // }
        // self.screen_start
        //     .set(self.screen_start.get().saturating_sub(lines * W));
        // self.screen_position.set(self.screen_start.get());
    }

    fn update(&self, screen: &mut [ScreenChar]) {
        if self.screen_position.get() >= W * H {
            self.scroll_down(1);
            return;
        }
        for char in &self.buffer.lock().as_ref()
            [self.read_position.get()..self.cursor.get()]
        {
            match char.char {
                Char::Backspace | Char::Delete => {
                    self.backspace();
                }
                Char::LineFeed => {}
                _ => {}
            }

            self.change_cursor_position_on_screen();
        }
    }

    fn write_cursor_position(&self) -> usize {
        self.screen_position.get()
    }

    fn set_cursor_position(&self, position: usize) {
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

    fn write_vga_char(&self, char: ScreenChar) {
        self.buffer.as_mut()[self.cursor.get()] = char;
        self.cursor.set(self.cursor.get() + 1);
    }

    fn set_color(&self, color: Option<ColorCode>) {
        self.color = color.unwrap_or_default();
    }

    fn color(&self) -> Option<ColorCode> {
        Some(self.color)
    }

    fn write_char(&self, char: char) {
        let c = char.as_ascii().expect("Entered invalid ascii character");
        self.write_vga_char(ScreenChar::new(
            c,
            self.color().unwrap_or_default(),
        ));
    }
}
