extern crate alloc;

use alloc::boxed::Box;

use common::constants::REGULAR_PAGE_SIZE;
use core::{ascii::Char, cell::Cell};

use crate::{
    color_code::ColorCode, generic_writer::GenericWriter,
    screen_char::ScreenChar,
};

#[derive(Debug)]
pub struct AdvancedWriter<const W: usize, const H: usize> {
    pub color: ColorCode,
    pub screen_start: Cell<usize>,
    pub cursor: Cell<usize>,
    pub screen_position: Cell<usize>,
    pub buffer: Box<[ScreenChar]>,
    pub backing: &'static mut [ScreenChar],
}

impl<const W: usize, const H: usize> Default for AdvancedWriter<W, H> {
    fn default() -> Self {
        const BUFFER_SIZE: usize =
            (10 * REGULAR_PAGE_SIZE) / size_of::<ScreenChar>();

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
            screen_start: Cell::new(0),
            screen_position: Cell::new(0),
            cursor: Cell::new(0),
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

    fn scroll_down(&self, lines: usize) {
        #[cfg(feature = "host")]
        {
            extern crate std;
            std::println!("Scroll Down Function Call");
        }
        if self.screen_position.get() < W * H {
            return;
        }
        self.screen_start
            .set(self.screen_start.get().saturating_add(lines * W));
        self.screen_position.set(self.screen_start.get());
    }

    fn scroll_up(&self, lines: usize) {
        if self.screen_start.get() > 0 {
            return;
        }
        self.screen_start
            .set(self.screen_start.get().saturating_sub(lines * W));
        self.screen_position.set(self.screen_start.get());
    }

    fn update(&mut self) {
        #[cfg(feature = "host")]
        {
            extern crate std;
            std::println!("Update Function Call");
        }

        if self.screen_position.get() - self.screen_start.get() >= W * H {
            self.scroll_down(1);
            return;
        }
        for char in &self.buffer.as_ref()[self.screen_position.get()
            ..self.cursor.get().min(self.screen_position.get() + W * H)]
        {
            match char.char {
                Char::Backspace | Char::Delete => {
                    self.screen_position
                        .set(self.screen_position.get().saturating_sub(1));

                    self.backing[self.screen_position.get()
                        - self.screen_start.get()] = ScreenChar::default();
                }
                Char::LineFeed => {
                    self.screen_position.set(
                        self.screen_position.get()
                            + (W - (self.screen_position.get() % W)),
                    );
                    if self.screen_position.get() - self.screen_start.get()
                        >= W * H
                    {
                        self.scroll_down(1);
                        return;
                    }
                    #[cfg(feature = "host")]
                    {
                        extern crate std;
                        std::println!(
                            "Screen Position: {}",
                            self.screen_position.get()
                        );
                    }

                    // self.cursor.set(self.screen_position.get());
                }
                _ => {
                    if self.screen_position.get() - self.screen_start.get()
                        >= W * H
                    {
                        self.scroll_down(1);
                        return;
                    }
                    self.backing[self.screen_position.get()
                        - self.screen_start.get()] = *char;

                    self.screen_position
                        .set(self.screen_position.get().saturating_add(1));
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
        if self.screen_start.get() != self.screen_position.get() {
            self.buffer
                [self.screen_start.get()..self.screen_position.get()]
                .copy_from_slice(
                    &self.backing[self.screen_start.get()
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
