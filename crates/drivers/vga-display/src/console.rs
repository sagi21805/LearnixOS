use core::{ascii::Char, fmt, io, ops::RangeBounds};

use common::{
    ansi::{BRIGHT_YELLOW, RED, RESET},
    enums::{Port, VgaCommand},
};
use sync::mutex::SpinMutex;
use x86::instructions::port::PortExt;

use crate::{
    ansi::{AnsiDecoder, Decoded},
    row_history::RowHistory,
    screen_char::ScreenChar,
    scrollback::Scrollback,
    vga_memory::VgaMemory,
};

/// Columns of the VGA text mode.
pub const WIDTH: usize = 80;
/// Rows of the VGA text mode.
pub const HEIGHT: usize = 25;
pub(crate) const CELLS: usize = WIDTH * HEIGHT;

/// The only screen, which every [`Console`] borrows.
///
/// Lock this before the scrollback, whenever taking both.
pub(crate) static SCREEN: SpinMutex<Screen> =
    // SAFETY: this is the only `VgaMemory` the kernel creates.
    SpinMutex::new(Screen::new(unsafe { VgaMemory::new() }));

/// The state of the VGA text screen, which outlives every [`Console`].
///
/// `cursor` always stays inside VGA text memory, because writing past the
/// last cell scrolls the screen first.
pub(crate) struct Screen {
    cursor: usize,
    decoder: AnsiDecoder,
    vga: VgaMemory,
}

impl Screen {
    const fn new(vga: VgaMemory) -> Self {
        Self {
            cursor: 0,
            decoder: AnsiDecoder::default(),
            vga,
        }
    }

    /// The rows above the cursor.
    pub(crate) fn finished_rows(&mut self) -> &[[ScreenChar; WIDTH]] {
        &self.vga.rows()[..self.cursor / WIDTH]
    }
}

/// The screen and whatever keeps its rows, as a writer that understands
/// ANSI colors, for the length of one operation.
pub(crate) struct Console<'a, H: RowHistory> {
    screen: &'a mut Screen,
    history: &'a mut H,
}

impl<'a, H: RowHistory> Console<'a, H> {
    pub(crate) fn new(screen: &'a mut Screen, history: &'a mut H) -> Self {
        Self { screen, history }
    }

    /// Prints `args` and moves the hardware cursor after them.
    pub(crate) fn print(&mut self, args: fmt::Arguments<'_>) {
        io::Write::write_fmt(self, args)
            .expect("only a failing Display implementation fails a print");
        io::Write::flush(self).expect("flushing the console can't fail");
    }

    /// Prints a failure message in color, from any state the panic left
    /// this console in.
    pub(crate) fn panic_message(&mut self, message: fmt::Arguments<'_>) {
        self.screen.decoder = AnsiDecoder::default();
        self.history.leave_view(&mut self.screen.vga);
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
        let (position, color) =
            (self.screen.cursor, self.screen.decoder.color());
        self.screen.vga.cells()[position] =
            ScreenChar::new(ascii.to_u8(), color);
        self.move_cursor_to(position + 1);
    }

    /// Blanks the rest of the row and starts the next one.
    fn line_feed(&mut self) {
        let position = self.screen.cursor;
        let next_row = (position / WIDTH + 1) * WIDTH;
        self.blank_cells(position..next_row);
        self.move_cursor_to(next_row);
    }

    /// Deletes the character before the cursor, or the line break if the
    /// cursor is at the start of a row.
    fn backspace(&mut self) {
        if self.screen.cursor.is_multiple_of(WIDTH) {
            self.reopen_previous_row();
        } else {
            self.erase_previous_cell();
        }
    }

    /// Moves the cursor back to the end of the previous row's text, which
    /// makes that row unfinished again.
    ///
    /// A full row wrapped without a line break, so this deletes its last
    /// character instead. When no row was kept, this does nothing.
    fn reopen_previous_row(&mut self) {
        self.scroll_contents_down();
        let Some(previous_row) =
            (self.screen.cursor / WIDTH).checked_sub(1)
        else {
            return;
        };
        let Some(span) = self.history.pop_row() else {
            return;
        };
        let text_end = span.columns().end;
        self.screen.cursor = previous_row * WIDTH + text_end;
        if text_end == WIDTH {
            self.erase_previous_cell();
        }
    }

    /// Blanks the cell before the cursor and moves back onto it.
    fn erase_previous_cell(&mut self) {
        self.screen.cursor -= 1;
        self.screen.vga.cells()[self.screen.cursor] =
            ScreenChar::default();
    }

    /// Moves the cursor, storing the row it finished if `position` starts
    /// a new one, and scrolling if `position` is past the last cell.
    fn move_cursor_to(&mut self, position: usize) {
        self.screen.cursor = position;
        if position.is_multiple_of(WIDTH) {
            let finished_row = position / WIDTH - 1;
            self.history.push_row(&self.screen.vga.rows()[finished_row]);
        }
        if position == CELLS {
            self.scroll_contents_up();
        }
    }

    /// Moves every row up by one and blanks the last row.
    fn scroll_contents_up(&mut self) {
        self.history.advance_screen_top();
        self.screen.vga.cells().copy_within(WIDTH.., 0);
        self.blank_cells(CELLS - WIDTH..);
        self.screen.cursor -= WIDTH;
    }

    /// Moves every row down by one and brings back the row above the
    /// screen, if there is one.
    fn scroll_contents_down(&mut self) {
        if self.history.has_rows_above_screen() {
            self.screen.vga.cells().copy_within(..CELLS - WIDTH, WIDTH);
            self.history
                .restore_row_above_screen(&mut self.screen.vga.rows()[0]);
            self.screen.cursor += WIDTH;
        }
    }

    fn blank_cells<R: RangeBounds<usize>>(&mut self, cells: R) {
        let range = core::slice::range(cells, ..CELLS);
        self.screen.vga.cells()[range].fill(ScreenChar::default());
    }

    /// Moves the blinking cursor, through CRTC registers `0xE` and `0xF`.
    fn move_hardware_cursor(&self) {
        // SAFETY: these registers only select where the cursor shows.
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.screen.cursor & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData.outb(((self.screen.cursor >> 8) & 0xff) as u8);
        }
    }
}

// Only the console with a scrollback can scroll its view.
impl Console<'_, Scrollback> {
    /// Moves the view `lines` rows up.
    pub(crate) fn scroll_view_up(&mut self, lines: usize) {
        let cursor_row = self.screen.cursor / WIDTH;
        self.history.scroll_view_up(
            &mut self.screen.vga,
            cursor_row,
            lines,
        );
    }

    /// Moves the view `lines` rows down.
    pub(crate) fn scroll_view_down(&mut self, lines: usize) {
        self.history.scroll_view_down(&mut self.screen.vga, lines);
    }
}

impl<H: RowHistory> io::Write for Console<'_, H> {
    /// New text always goes to the live screen, so this leaves the view
    /// first.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.history.leave_view(&mut self.screen.vga);
        for &byte in bytes {
            match self.screen.decoder.advance(byte) {
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
