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

/// The only screen, which every [`Console`] borrows.
pub(crate) static SCREEN: SpinMutex<Screen> =
    // SAFETY: this is the only `VgaMemory` the kernel creates.
    SpinMutex::new(Screen::new(unsafe { VgaMemory::new() }));

/// The state of the VGA text screen, which outlives every [`Console`].
///
/// `cursor` always stays inside VGA text memory, because moving past the
/// last cell shifts the text up first.
pub(crate) struct Screen {
    pub(crate) cursor: usize,
    decoder: AnsiDecoder,
    pub(crate) vga: VgaMemory,
    pub(crate) history: &'static mut dyn RowHistory,
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
    pub(crate) fn attach_scrollback<A: Allocator + 'static>(
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
}

/// The screen as a writer that understands ANSI colors, for the length of
/// one operation.
pub(crate) struct Console<'a> {
    screen: &'a mut Screen,
}

impl<'a> Console<'a> {
    pub(crate) fn new(screen: &'a mut Screen) -> Self { Self { screen } }

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

    /// Shows older text, for the scroll keys.
    pub(crate) fn scroll_view_up(&mut self, lines: usize) {
        let cursor_row = self.screen.cursor / WIDTH;
        self.screen.history.scroll_view_up(
            &mut self.screen.vga,
            cursor_row,
            lines,
        );
    }

    /// Shows newer text, for the scroll keys.
    pub(crate) fn scroll_view_down(&mut self, lines: usize) {
        self.screen
            .history
            .scroll_view_down(&mut self.screen.vga, lines);
    }

    /// Writes one character in the current color.
    fn put(&mut self, ascii: Char) {
        let (position, color) =
            (self.screen.cursor, self.screen.decoder.color());
        self.screen.vga.cells()[position] =
            ScreenChar::new(ascii.to_u8(), color);
        self.move_cursor_to(position + 1);
    }

    /// Ends the line, so the next text starts on a new row.
    fn line_feed(&mut self) {
        let position = self.screen.cursor;
        let next_row = (position / WIDTH + 1) * WIDTH;
        self.screen.vga.blank_cells(position..next_row);
        self.move_cursor_to(next_row);
    }

    /// Deletes the character before the cursor, or the line break if the
    /// cursor is at the start of a screen row.
    fn backspace(&mut self) {
        if self.screen.cursor.is_multiple_of(WIDTH) {
            self.reopen_previous_row();
        } else {
            self.erase_previous_cell();
        }
    }

    /// Undoes a line break, so backspace can go on deleting the previous
    /// row.
    fn reopen_previous_row(&mut self) {
        self.shift_output_down();
        let (Some(prev_row), Some(span)) = (
            (self.screen.cursor / WIDTH).checked_sub(1),
            self.screen.history.pop_row(),
        ) else {
            return;
        };

        let column = span.columns().end.min(WIDTH - 1);
        self.screen.cursor = prev_row * WIDTH + column;
        self.screen.vga.cells()[self.screen.cursor] =
            ScreenChar::default();
    }

    #[inline]
    /// Deletes the character before the cursor.
    fn erase_previous_cell(&mut self) {
        self.screen.cursor -= 1;
        self.screen.vga.cells()[self.screen.cursor] =
            ScreenChar::default();
    }

    /// Move the cursor into a given position. If the position is the start
    /// of a new row, it saves the previous row. If the position is the end
    /// of the screen, then it shifts it content up.
    fn move_cursor_to(&mut self, position: usize) {
        self.screen.cursor = position;
        if position.is_multiple_of(WIDTH) {
            let finished_row = (position / WIDTH) - 1;
            self.screen
                .history
                .push_row(&self.screen.vga.rows()[finished_row]);
            if position == CELLS {
                self.shift_output_up();
            }
        }
    }

    /// Makes room for a new row once the screen is full.
    fn shift_output_up(&mut self) {
        self.screen.history.shift_output_up(&mut self.screen.vga);
        self.screen.cursor -= WIDTH;
    }

    /// Keeps the screen full when backspace reopens a previous row, if a
    /// kept row can fill it.
    fn shift_output_down(&mut self) {
        if self.screen.history.shift_output_down(&mut self.screen.vga) {
            self.screen.cursor += WIDTH;
        }
    }

    /// Shows the blinking cursor where the next character goes, through
    /// CRTC registers `0xE` and `0xF`.
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

impl io::Write for Console<'_> {
    /// New text always goes to the output area, so this moves the view
    /// back to it first.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.scroll_view_down(usize::MAX);
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
