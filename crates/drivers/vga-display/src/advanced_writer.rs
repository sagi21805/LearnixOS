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
    pub line_offset: usize,
    pub buffer: Box<[ScreenChar]>,
    pub display_line: usize,
    pub row_table: Box<[usize]>,
    pub screen: &'static SpinMutex<Screen>,
}

impl<const W: usize, const H: usize> Default for AdvancedWriter<W, H> {
    fn default() -> Self {
        const BUFFER_SIZE: usize =
            (64 * REGULAR_PAGE_SIZE) / size_of::<ScreenChar>();

        const ROW_TABLE_SIZE: usize =
            (4 * REGULAR_PAGE_SIZE) / size_of::<usize>();

        Self {
            color: ColorCode::default(),
            screen_start: 0,
            cursor: 0,
            line: 1,
            line_offset: 0,
            display_line: 0,
            buffer: unsafe {
                Box::<[ScreenChar; BUFFER_SIZE]>::new_zeroed()
                    .assume_init()
            },
            row_table: unsafe {
                Box::<[usize; ROW_TABLE_SIZE]>::new_zeroed().assume_init()
            },
            screen: &SCREEN,
        }
    }
}

impl<const W: usize, const H: usize> GenericWriter
    for AdvancedWriter<W, H>
{
    fn backspace(&mut self) {
        self.write_vga_char(ScreenChar::new(Char::Backspace, self.color));
        if self.line_offset > 0 {
            self.line_offset -= 1;
        } else {
            self.line -= 1;
            self.line_offset = self.screen_width();
        }
    }

    fn new_line(&mut self) {
        self.write_vga_char(ScreenChar::new(Char::LineFeed, self.color));
        self.line += 1;
        if self.line >= self.row_table.len() {
            return;
        }
        self.row_table[self.line] = self.row_table[self.line - 1];
        self.line_offset = 0;
        self.scroll_down(1);
    }

    fn screen_height(&self) -> usize { H }

    fn screen_width(&self) -> usize { W }

    fn scroll_down(&mut self, lines: usize) {
        let new_line = self.display_line.saturating_add(lines);
        if new_line + self.screen_height() >= self.row_table.len() {
            return;
        }
        if self.row_table[new_line + self.screen_height()] != 0 {
            self.screen.lock().scroll_up(lines);
            self.screen.lock().reset_cursor();
            self.display_line = new_line;
            self.screen_start = self.row_table[new_line] as usize;
        }
    }

    fn scroll_up(&mut self, lines: usize) {
        let new_line = self.display_line.saturating_sub(lines);
        if new_line >= self.row_table.len() {
            return;
        }
        if self.row_table[new_line] != 0 || new_line == 0 {
            self.screen.lock().scroll_down(lines);
            self.screen.lock().reset_cursor();
            self.display_line = new_line;
            self.screen_start = self.row_table[new_line] as usize;
        }
    }

    fn update(&mut self) {
        for i in self.screen_start..self.cursor {
            let mut screen = self.screen.lock();
            let char = self.buffer.get(i).cloned().unwrap_or_default();
            match screen.write_char(char) {
                WriteInfo::LineDown
                | WriteInfo::LineUp
                | WriteInfo::SameLine => {}
                WriteInfo::EndOfScreen => break,
            }
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
        if self.cursor + 1 > self.buffer.len() {
            return;
        }

        let position = &mut self.buffer.as_mut()[self.cursor];
        unsafe { ::core::ptr::write_volatile(position as *mut _, char) };
        self.cursor += 1;
        self.line_offset += 1;
        if self.line + 1 >= self.row_table.len() {
            return;
        }
        self.row_table[self.line] += 1;
        if self.line_offset >= self.screen_width() {
            self.line += 1;
            self.line_offset = 0;
            self.row_table[self.line] = self.row_table[self.line - 1];
        }
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
