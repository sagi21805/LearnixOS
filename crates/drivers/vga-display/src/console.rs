use core::{ascii::Char, fmt, io, ptr::with_exposed_provenance_mut};

use common::{
    ansi::{BRIGHT_YELLOW, RED, RESET},
    constants::VGA_BUFFER_PTR,
    enums::{Port, VgaCommand},
    volatile::Volatile,
};
use sync::mutex::SpinMutex;
use x86::instructions::port::PortExt;

use crate::{
    ansi::{AnsiDecoder, Decoded},
    color_code::ColorCode,
    screen_char::ScreenChar,
};

/// Columns of the VGA text mode.
pub const WIDTH: usize = 80;
/// Rows of the VGA text mode.
pub const HEIGHT: usize = 25;
const CELLS: usize = WIDTH * HEIGHT;

/// The console behind every print macro.
pub static CONSOLE: SpinMutex<Console> = SpinMutex::new(Console::new());

/// The VGA text screen, as a writer that understands ANSI colors.
///
/// `cursor` always stays inside VGA text memory, because writing past the
/// last cell scrolls the screen first.
pub struct Console {
    cursor: usize,
    color: ColorCode,
    decoder: AnsiDecoder,
}

pub enum WriteInfo {
    /// Reached the end of the screen.
    EndOfScreen,
    /// Write was on the same line.
    SameLine,
    /// Went one line up
    LineUp,
    /// Went one line down
    LineDown,
}

impl Console {
    const fn new() -> Self {
        Self {
            cursor: 0,
            color: ColorCode::default(),
            decoder: AnsiDecoder::default(),
        }
    }

    /// Prints a failure message in color, from any state the panic left
    /// this console in.
    pub fn panic_message(&mut self, message: fmt::Arguments<'_>) {
        self.decoder = AnsiDecoder::default();
        // Nothing can report a failure to print while panicking.
        let _ = io::Write::write_fmt(
            self,
            format_args!(
                "{BRIGHT_YELLOW}\n[{RED} FAIL {BRIGHT_YELLOW}]: \
                 {message}{RESET}"
            ),
        );
        self.move_hardware_cursor();
    }

    /// VGA text memory, which `&mut self` keeps exclusive because
    /// [`CONSOLE`] is the only `Console`.
    pub(crate) fn vga_cells(
        &mut self,
    ) -> &mut [Volatile<ScreenChar>; CELLS] {
        let cells = with_exposed_provenance_mut(VGA_BUFFER_PTR as usize);
        // SAFETY: the boot page tables map VGA text memory, and any byte
        // is a valid `ScreenChar`.
        unsafe { &mut *cells }
    }

    /// Writes one glyph in the current color.
    fn put(&mut self, glyph: u8) {
        let (position, color) = (self.cursor, self.color);
        self.vga_cells()[position].write(ScreenChar::new(glyph, color));
        self.move_cursor_to(position + 1);
    }

    /// Blanks the rest of the row and starts the next one.
    fn line_feed(&mut self) {
        let position = self.cursor;
        let next_row = (position / WIDTH + 1) * WIDTH;
        self.blank_cells(position, next_row);
        self.move_cursor_to(next_row);
    }

    /// Blanks the cell before the cursor and moves back onto it.
    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.blank_cells(self.cursor, self.cursor + 1);
        }
    }

    /// Moves the cursor, scrolling if `position` is past the last cell.
    fn move_cursor_to(&mut self, position: usize) {
        self.cursor = position;
        if position == CELLS {
            self.scroll_contents_up();
        }
    }

    /// Moves every row up by one and blanks the last row.
    fn scroll_contents_up(&mut self) {
        let cells = self.vga_cells();
        for position in 0..CELLS - WIDTH {
            cells[position].write(cells[position + WIDTH].read());
        }
        self.blank_cells(CELLS - WIDTH, CELLS);
        self.cursor -= WIDTH;
    }

    fn blank_cells(&mut self, start: usize, end: usize) {
        self.vga_cells()[start..end]
            .iter_mut()
            .for_each(|cell| cell.write(ScreenChar::default()));
    }

    /// Moves the blinking cursor, through CRTC registers `0xE` and `0xF`.
    fn move_hardware_cursor(&self) {
        // SAFETY: these registers only select where the cursor shows.
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.cursor & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData.outb(((self.cursor >> 8) & 0xff) as u8);
        }
    }

    pub fn write_char(&mut self, c: ScreenChar) -> WriteInfo {
        let info = match Char::from_u8(c.glyph) {
            Some(Char::LineFeed) => self.legacy_new_line(),
            Some(Char::Backspace | Char::Delete) => {
                self.legacy_backspace()
            }
            _ => {
                if self.cursor < CELLS {
                    let position = self.cursor;
                    self.vga_cells()[position].write(c);
                    self.cursor += 1;
                    if self.cursor.is_multiple_of(WIDTH) {
                        WriteInfo::LineUp
                    } else {
                        WriteInfo::SameLine
                    }
                } else {
                    WriteInfo::EndOfScreen
                }
            }
        };
        if self.cursor < CELLS {
            self.move_hardware_cursor();
        }

        info
    }

    pub fn legacy_new_line(&mut self) -> WriteInfo {
        let offset = self.cursor % WIDTH;
        let new_position = self.cursor + (WIDTH - offset);

        self.blank_cells(self.cursor, new_position.min(CELLS));

        if new_position >= CELLS {
            self.cursor = CELLS - 1;
            WriteInfo::EndOfScreen
        } else {
            self.cursor = new_position;
            WriteInfo::LineUp
        }
    }

    pub fn legacy_backspace(&mut self) -> WriteInfo {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.blank_cells(self.cursor, self.cursor + 1);
            // Backspace to the first character of the line
            if self.cursor % WIDTH == WIDTH - 1 {
                return WriteInfo::LineDown;
            }
        }
        WriteInfo::SameLine
    }

    pub fn cursor(&self) -> usize { self.cursor }

    pub(crate) fn scroll_up(&mut self, lines: usize) {
        self.blank_cells(CELLS - lines * WIDTH, CELLS);
    }

    pub(crate) fn scroll_down(&mut self, lines: usize) {
        self.blank_cells(0, lines * WIDTH);
    }

    pub fn reset_cursor(&mut self) { self.cursor = 0; }

    /// Blanks the whole screen, and moves the cursor to its start.
    pub fn clear(&mut self) {
        self.blank_cells(0, CELLS);
        self.cursor = 0;
    }
}

impl io::Write for Console {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for &byte in bytes {
            match self.decoder.advance(byte) {
                Some(Decoded::Print(glyph)) => self.put(glyph),
                Some(Decoded::LineFeed) => self.line_feed(),
                Some(Decoded::Backspace) => self.backspace(),
                Some(Decoded::Rendition(params)) => {
                    self.color.apply_sgr(params)
                }
                None => {}
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.move_hardware_cursor();
        Ok(())
    }
}
