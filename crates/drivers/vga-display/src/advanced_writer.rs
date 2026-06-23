extern crate alloc;

use core::ascii::Char;

use alloc::boxed::Box;

use common::constants::REGULAR_PAGE_SIZE;
use sync::mutex::SpinMutex;

use crate::{
    SCREEN, Screen, WriteInfo, color_code::ColorCode,
    generic_writer::GenericWriter, screen_char::ScreenChar,
};

pub struct AdvancedWriter<const W: usize, const H: usize> {
    pub color: ColorCode,
    // Check if cursor, read position and buffer, can be a ring buffer
    pub cursor: usize,
    pub screen_start: usize,
    pub line: usize,
    pub buffer: Box<[ScreenChar]>,
    pub row_table: Box<[u16]>,
    pub screen: &'static SpinMutex<Screen>,
}

impl<const W: usize, const H: usize> Default for AdvancedWriter<W, H> {
    fn default() -> Self {
        const BUFFER_SIZE: usize =
            (2 * REGULAR_PAGE_SIZE) / size_of::<ScreenChar>();

        const ROW_TABLE_SIZE: usize =
            (REGULAR_PAGE_SIZE) / size_of::<u16>();

        Self {
            color: ColorCode::default(),
            screen_start: 0,
            cursor: 0,
            line: 0,
            buffer: unsafe {
                Box::<[ScreenChar; BUFFER_SIZE]>::new_zeroed()
                    .assume_init()
            },
            row_table: unsafe {
                Box::<[u16; ROW_TABLE_SIZE]>::new_zeroed().assume_init()
            },
            screen: &SCREEN,
        }
    }
}

impl<const W: usize, const H: usize> AdvancedWriter<W, H> {
    fn cursor_line(&self) -> usize { self.cursor / self.screen_width() }
}

impl<const W: usize, const H: usize> GenericWriter
    for AdvancedWriter<W, H>
{
    // TODO: IMPLEMENT THE LOGIC IF THE BACKSPACE GOES TO THE PREVIOUS LINE
    fn backspace(&mut self) {
        self.write_vga_char(ScreenChar::new(Char::Backspace, self.color));
    }

    fn new_line(&mut self) {
        self.write_vga_char(ScreenChar::new(Char::LineFeed, self.color));
        self.row_table[self.cursor_line() + 1] = self.cursor as u16;
    }

    fn screen_height(&self) -> usize { H }

    fn screen_width(&self) -> usize { W }

    fn scroll_down(&mut self, lines: usize) {
        let new_line = self.line.saturating_sub(lines);
        if self.row_table[new_line] != 0 {
            self.line = new_line;
            self.screen_start = self.row_table[new_line] as usize;
            self.screen.lock().scroll_down(lines);
        }
    }

    fn scroll_up(&mut self, lines: usize) {
        let new_line = self.line.saturating_add(lines);
        if self.row_table[new_line] != 0 {
            self.line = new_line;
            self.screen_start = self.row_table[new_line] as usize;
            self.screen.lock().scroll_up(lines);
        }
    }

    fn update(&mut self) {
        let mut screen = self.screen.lock();
        for i in self.screen_start..self.cursor {
            let char = self.buffer.get(i).cloned().unwrap_or_default();
            match screen.write_char(char) {
                WriteInfo::SameLine => {
                    // Change of current line offset is done at the end of
                    // the function because it is also relevant for line up
                    // and line down.
                }
                WriteInfo::LineUp => self.line += 1,
                WriteInfo::LineDown => self.line -= 1,
                WriteInfo::EndOfScreen => break,
            }
            self.row_table[self.line] = self.cursor as u16;
            self.screen_start += 1;
        }
    }

    fn write_cursor_position(&self) -> usize { todo!() }

    fn set_cursor_position(&mut self, position: usize) {
        let mut screen = self.screen.lock();
        if self.screen_start == 0 {
            for i in 0..position {
                self.write_vga_char(screen.buffer[i]);
            }
            screen.reset_cursor();
            // self.screen_start = self.cursor;
            // self.line = self.cursor / screen.width;
        }
    }

    fn write_vga_char(&mut self, char: ScreenChar) {
        self.buffer.as_mut()[self.cursor] = char;
        self.cursor += 1;
    }

    fn set_color(&mut self, color: ColorCode) { self.color = color; }

    fn color(&self) -> ColorCode { self.color }

    fn write_char(&mut self, char: char) {
        let ascii =
            char.as_ascii().expect("Entered invalid ascii character");

        match ascii {
            Char::LineFeed => {
                self.new_line();
            }
            Char::Delete | Char::Backspace => {
                self.backspace();
            }
            _ => {
                if !ascii.is_control() {
                    self.write_vga_char(ScreenChar {
                        char: ascii,
                        color_code: self.color,
                    });
                }
            }
        }
    }
}
