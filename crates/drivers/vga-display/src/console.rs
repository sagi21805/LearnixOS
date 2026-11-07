use core::{
    array,
    ascii::Char,
    fmt, io,
    ops::{Range, RangeBounds},
    ptr,
    range::RangeFrom,
};

use common::{
    ansi::{BRIGHT_YELLOW, RED, RESET},
    constants::VGA_BUFFER_PTR,
    enums::{Port, VgaCommand},
};
use sync::mutex::SpinMutex;
use x86::instructions::port::PortExt;

use crate::{
    ansi::{AnsiDecoder, Decoded},
    screen_char::ScreenChar,
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

    /// Starts keeping the rows that leave the top of the screen.
    pub fn attach_scrollback(&mut self, scrollback: Scrollback) {
        self.scrollback = Some(scrollback);
    }

    /// Shows rows `lines` older than the current view.
    ///
    /// The first step back stores the live rows as spans too, so the whole
    /// view comes from [`Scrollback`].
    pub fn scroll_view_up(&mut self, lines: usize) {
        let Some(scrollback) = &self.scrollback else {
            return;
        };
        if lines == 0 || !scrollback.has_history() {
            return;
        }
        if !scrollback.is_viewing_history() {
            self.store_live_rows();
        }
        if let Some(scrollback) = &mut self.scrollback {
            scrollback.view_older(lines);
        }
        self.draw_view();
    }

    /// Shows rows `lines` newer than the current view.
    ///
    /// Reaching the bottom draws the live rows back, then removes their
    /// spans, so they aren't stored twice when they later leave the
    /// screen.
    pub fn scroll_view_down(&mut self, lines: usize) {
        let Some(scrollback) = &mut self.scrollback else {
            return;
        };
        if !scrollback.is_viewing_history() {
            return;
        }
        scrollback.view_newer(lines);
        self.draw_view();
        if let Some(scrollback) = &mut self.scrollback
            && !scrollback.is_viewing_history()
        {
            scrollback.forget_newest_rows(HEIGHT);
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

    /// VGA text memory, which `&mut self` keeps exclusive because
    /// [`CONSOLE`] is the only `Console`.
    fn vga_cells(&mut self) -> &mut [ScreenChar; CELLS] {
        let cells =
            ptr::with_exposed_provenance_mut(VGA_BUFFER_PTR as usize);
        // SAFETY: the boot page tables map VGA text memory, and any byte
        // is a valid `ScreenChar`.
        unsafe { &mut *cells }
    }

    /// Writes one character in the current color.
    fn put(&mut self, ascii: Char) {
        let (position, color) = (self.cursor, self.decoder.color());
        self.vga_cells()[position] = ScreenChar::new(ascii.to_u8(), color);
        self.move_cursor_to(position + 1);
    }

    /// Blanks the rest of the row and starts the next one.
    fn line_feed(&mut self) {
        let position = self.cursor;
        let next_row = (position / WIDTH + 1) * WIDTH;
        self.blank_cells(position..next_row);
        self.move_cursor_to(next_row);
    }

    /// Blanks the cell before the cursor and moves back onto it.
    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.blank_cells(self.cursor..self.cursor + 1);
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
        self.store_row(0);
        self.vga_cells().copy_within(WIDTH.., 0);
        self.blank_cells(CELLS - WIDTH..);
        self.cursor -= WIDTH;
    }

    /// Copies every live row into the scrollback.
    fn store_live_rows(&mut self) {
        for row in 0..HEIGHT {
            self.store_row(row);
        }
    }

    /// Copies one row of the screen into the scrollback, if there is one.
    fn store_row(&mut self, row: usize) {
        let Some(scrollback) = &mut self.scrollback else {
            return;
        };
        let cells = self.vga.cells();
        let row = array::from_fn(|column| cells[row * WIDTH + column]);
        scrollback.push_row(&row);
    }

    /// Draws the rows of the scrollback's current view.
    fn draw_view(&mut self) {
        for screen_row in 0..HEIGHT {
            let Some(row) = self
                .scrollback
                .as_ref()
                .map(|scrollback| scrollback.visible_row(screen_row))
            else {
                return;
            };
            let cells = self.vga_cells();
            for (column, cell) in row.into_iter().enumerate() {
                cells[screen_row * WIDTH + column] = cell;
            }
        }
    }

    fn blank_cells<R: RangeBounds<usize>>(&mut self, cells: R) {
        let range = core::slice::range(cells, ..CELLS);
        self.vga_cells()[range].fill(ScreenChar::default());
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
        self.blank_cells(0..);
        self.cursor = 0;
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
struct VgaMemory;

impl VgaMemory {
    fn cells(&mut self) -> &mut [ScreenChar; CELLS] {
        let cells =
            ptr::with_exposed_provenance_mut(VGA_BUFFER_PTR as usize);
        // SAFETY: the boot page tables map VGA text memory, and any byte
        // is a valid `ScreenChar`.
        unsafe { &mut *cells }
    }
}
