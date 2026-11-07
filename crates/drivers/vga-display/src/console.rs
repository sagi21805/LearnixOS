use core::{ascii::Char, fmt, io, ops::RangeBounds, ptr};

use common::{
    ansi::{BRIGHT_YELLOW, RED, RESET},
    constants::VGA_BUFFER_PTR,
    enums::{Port, VgaCommand},
};
use sync::mutex::SpinMutex;
use x86::instructions::port::PortExt;

use crate::{
    ansi::{AnsiDecoder, Decoded},
    screen_char::{RowText, ScreenChar},
    scrollback::Scrollback,
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
    decoder: AnsiDecoder,
    vga: VgaMemory,
    scrollback: Option<Scrollback>,
}

impl Console {
    const fn new() -> Self {
        Self {
            cursor: 0,
            decoder: AnsiDecoder::default(),
            scrollback: None,
            vga: VgaMemory,
        }
    }

    /// Starts keeping finished rows, beginning with those above the
    /// cursor.
    pub fn attach_scrollback(&mut self, mut scrollback: Scrollback) {
        for row in &self.vga.rows()[..self.cursor / WIDTH] {
            scrollback.push_row(row);
        }
        self.scrollback = Some(scrollback);
    }

    /// Moves the scrollback's view `lines` rows up.
    pub fn scroll_view_up(&mut self, lines: usize) {
        if let Some(scrollback) = &mut self.scrollback {
            scrollback.scroll_view_up(
                &mut self.vga,
                self.cursor / WIDTH,
                lines,
            );
        }
    }

    /// Moves the scrollback's view `lines` rows down.
    pub fn scroll_view_down(&mut self, lines: usize) {
        if let Some(scrollback) = &mut self.scrollback {
            scrollback.scroll_view_down(&mut self.vga, lines);
        }
    }

    /// Prints a failure message in color, from any state the panic left
    /// this console in.
    pub fn panic_message(&mut self, message: fmt::Arguments<'_>) {
        self.decoder = AnsiDecoder::default();
        self.scroll_view_down(usize::MAX);
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

    /// Writes one character in the current color.
    fn put(&mut self, ascii: Char) {
        let (position, color) = (self.cursor, self.decoder.color());
        self.vga.cells()[position] = ScreenChar::new(ascii.to_u8(), color);
        self.move_cursor_to(position + 1);
    }

    /// Blanks the rest of the row and starts the next one.
    fn line_feed(&mut self) {
        let position = self.cursor;
        let next_row = (position / WIDTH + 1) * WIDTH;
        self.blank_cells(position..next_row);
        self.move_cursor_to(next_row);
    }

    /// Deletes the character before the cursor, or the line break if the
    /// cursor is at the start of a row.
    fn backspace(&mut self) {
        if self.cursor.is_multiple_of(WIDTH) {
            self.reopen_previous_row();
        } else {
            self.erase_previous_cell();
        }
    }

    /// Moves the cursor back to the end of the previous row's text, which
    /// makes that row unfinished again.
    ///
    /// A full row wrapped without a line break, so this deletes its last
    /// character instead.
    fn reopen_previous_row(&mut self) {
        self.scroll_contents_down();
        let Some(previous_row) = (self.cursor / WIDTH).checked_sub(1)
        else {
            return;
        };
        if let Some(scrollback) = &mut self.scrollback {
            scrollback.pop_row();
        }
        let text_end = self.vga.rows()[previous_row].text_columns().end;
        self.cursor = previous_row * WIDTH + text_end;
        if text_end == WIDTH {
            self.erase_previous_cell();
        }
    }

    /// Blanks the cell before the cursor and moves back onto it.
    fn erase_previous_cell(&mut self) {
        self.cursor -= 1;
        self.vga.cells()[self.cursor] = ScreenChar::default();
    }

    /// Moves the cursor, storing the row it finished if `position` starts
    /// a new one, and scrolling if `position` is past the last cell.
    fn move_cursor_to(&mut self, position: usize) {
        self.cursor = position;
        if position.is_multiple_of(WIDTH)
            && let Some(scrollback) = &mut self.scrollback
        {
            scrollback.push_row(&self.vga.rows()[position / WIDTH - 1]);
        }
        if position == CELLS {
            self.scroll_contents_up();
        }
    }

    /// Moves every row up by one and blanks the last row.
    fn scroll_contents_up(&mut self) {
        if let Some(scrollback) = &mut self.scrollback {
            scrollback.advance_screen_top();
        }
        self.vga.cells().copy_within(WIDTH.., 0);
        self.blank_cells(CELLS - WIDTH..);
        self.cursor -= WIDTH;
    }

    /// Moves every row down by one and brings back the row above the
    /// screen, if there is one.
    fn scroll_contents_down(&mut self) {
        if let Some(scrollback) = &mut self.scrollback
            && scrollback.has_rows_above_screen()
        {
            self.vga.cells().copy_within(..CELLS - WIDTH, WIDTH);
            scrollback.restore_row_above_screen(&mut self.vga.rows()[0]);
            self.cursor += WIDTH;
        }
    }

    fn blank_cells<R: RangeBounds<usize>>(&mut self, cells: R) {
        let range = core::slice::range(cells, ..CELLS);
        self.vga.cells()[range].fill(ScreenChar::default());
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

    /// Blanks the whole screen, and moves the cursor to its start.
    pub fn clear(&mut self) {
        self.scroll_view_down(usize::MAX);
        self.blank_cells(0..);
        self.cursor = 0;
        if let Some(scrollback) = &mut self.scrollback {
            scrollback.start_new_screen();
        }
    }
}

impl io::Write for Console {
    /// New text always goes to the live screen, so this leaves the
    /// scrollback's view first.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.scroll_view_down(usize::MAX);
        for &byte in bytes {
            match self.decoder.advance(byte) {
                Some(Decoded::Print(ascii)) => self.put(ascii),
                Some(Decoded::LineFeed) => self.line_feed(),
                Some(Decoded::Backspace) => self.backspace(),
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

/// Exclusive access to VGA text memory.
pub(crate) struct VgaMemory;

impl VgaMemory {
    pub(crate) fn cells(&mut self) -> &mut [ScreenChar; CELLS] {
        let cells =
            ptr::with_exposed_provenance_mut(VGA_BUFFER_PTR as usize);
        // SAFETY: the boot page tables map VGA text memory, and any byte
        // is a valid `ScreenChar`.
        unsafe { &mut *cells }
    }

    pub(crate) fn rows(&mut self) -> &mut [[ScreenChar; WIDTH]] {
        self.cells().as_chunks_mut().0
    }
}
