use alloc::boxed::Box;
use core::{alloc::Allocator, ascii::Char, fmt, io};

use common::{
    ansi::{BRIGHT_YELLOW, RED, RESET},
    enums::{Port, VgaCommand},
};
use sync::mutex::SpinMutex;
use x86::instructions::port::PortExt;

use crate::{
    ansi::{AnsiDecoder, Decoded},
    row_history::{NoHistory, RowHistory},
    screen_char::ScreenChar,
    scrollback::Scrollback,
    vga_memory::VgaMemory,
};

/// Columns of the VGA text mode.
pub const WIDTH: usize = 80;
/// Rows of the VGA text mode.
pub const HEIGHT: usize = 25;
pub(crate) const CELLS: usize = WIDTH * HEIGHT;

/// The only screen. Every operation locks it for its whole length.
pub static SCREEN: SpinMutex<Screen> =
    // SAFETY: this is the only `VgaMemory` the kernel creates.
    SpinMutex::new(Screen::new(unsafe { VgaMemory::new() }));

/// The VGA text screen, as a writer that understands ANSI colors.
///
/// `cursor` always stays inside VGA text memory, because moving past the
/// last cell shifts the text up first.
pub struct Screen {
    cursor: usize,
    decoder: AnsiDecoder,
    vga: VgaMemory,
    history: &'static mut dyn RowHistory,
}

impl Screen {
    const fn new(vga: VgaMemory) -> Self {
        Self {
            cursor: 0,
            decoder: AnsiDecoder::default(),
            vga,
            history: NoHistory::leaked(),
        }
    }

    /// Starts keeping finished rows in `scrollback`, beginning with those
    /// above the cursor.
    ///
    /// A second call drops the rows the first scrollback kept.
    pub fn attach_scrollback<A: Allocator + 'static>(
        &mut self,
        scrollback: Box<Scrollback, A>,
    ) {
        let scrollback = Box::leak(scrollback);
        let cursor_row = self.cursor / WIDTH;
        for row in &self.vga.rows()[..cursor_row] {
            scrollback.push_row(row);
        }
        self.history = scrollback;
    }

    /// Prints `args` and moves the hardware cursor after them.
    pub(crate) fn print(&mut self, args: fmt::Arguments<'_>) {
        io::Write::write_fmt(self, args)
            .expect("only a failing Display implementation fails a print");
        io::Write::flush(self).expect("flushing the console can't fail");
    }

    /// Prints a failure message in color, from any state the panic left
    /// the screen in.
    pub(crate) fn panic_message(&mut self, message: fmt::Arguments<'_>) {
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

    /// Shows older text, for reading what left the screen. Until
    /// [`Self::attach_scrollback`], no rows are kept, so this does
    /// nothing.
    pub fn scroll_view_up(&mut self, lines: usize) {
        let cursor_row = self.cursor / WIDTH;
        self.history
            .scroll_view_up(&mut self.vga, cursor_row, lines);
    }

    /// Shows newer text, on the way back from reading history.
    pub fn scroll_view_down(&mut self, lines: usize) {
        self.history.scroll_view_down(&mut self.vga, lines);
    }

    /// Writes one character in the current color.
    fn put(&mut self, ascii: Char) {
        let (position, color) = (self.cursor, self.decoder.color());
        self.vga.cells()[position] = ScreenChar::new(ascii.to_u8(), color);
        self.move_cursor_to(position + 1);
    }

    /// Ends the line, so the next text starts on a new row.
    fn line_feed(&mut self) {
        let position = self.cursor;
        let next_row = (position / WIDTH + 1) * WIDTH;
        self.vga.blank_cells(position..next_row);
        self.move_cursor_to(next_row);
    }

    /// Deletes the character before the cursor, or the line break if the
    /// cursor is at the start of a screen row.
    fn backspace(&mut self) {
        if self.cursor.is_multiple_of(WIDTH) {
            self.reopen_previous_row();
        } else {
            self.erase_previous_cell();
        }
    }

    /// Undoes a line break, so backspace can go on deleting the previous
    /// row.
    fn reopen_previous_row(&mut self) {
        self.shift_output_down();
        let (Some(prev_row), Some(span)) =
            ((self.cursor / WIDTH).checked_sub(1), self.history.pop_row())
        else {
            return;
        };

        let column = span.columns().end.min(WIDTH - 1);
        self.cursor = prev_row * WIDTH + column;
        self.vga.cells()[self.cursor] = ScreenChar::default();
    }

    #[inline]
    /// Deletes the character before the cursor.
    fn erase_previous_cell(&mut self) {
        self.cursor -= 1;
        self.vga.cells()[self.cursor] = ScreenChar::default();
    }

    /// Move the cursor into a given position. If the position is the start
    /// of a new row, it saves the previous row. If the position is the end
    /// of the screen, then it shifts it content up.
    fn move_cursor_to(&mut self, position: usize) {
        self.cursor = position;
        if position.is_multiple_of(WIDTH) {
            let finished_row = (position / WIDTH) - 1;
            self.history.push_row(&self.vga.rows()[finished_row]);
            if position == CELLS {
                self.shift_output_up();
            }
        }
    }

    /// Makes room for a new row once the screen is full.
    fn shift_output_up(&mut self) {
        self.history.shift_output_up(&mut self.vga);
        self.cursor -= WIDTH;
    }

    /// Keeps the screen full when backspace reopens a previous row, if a
    /// kept row can fill it.
    fn shift_output_down(&mut self) {
        if self.history.shift_output_down(&mut self.vga) {
            self.cursor += WIDTH;
        }
    }

    /// Shows the blinking cursor where the next character goes, through
    /// CRTC registers `0xE` and `0xF`.
    fn move_hardware_cursor(&self) {
        // SAFETY: these registers only select where the cursor shows.
        unsafe {
            Port::VgaControl.outb(VgaCommand::CursorOffsetLow as u8);
            Port::VgaData.outb((self.cursor & 0xff) as u8);
            Port::VgaControl.outb(VgaCommand::CursorOffsetHigh as u8);
            Port::VgaData.outb(((self.cursor >> 8) & 0xff) as u8);
        }
    }
}

impl io::Write for Screen {
    /// New text always goes to the output area, so this moves the view
    /// back to it first.
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
