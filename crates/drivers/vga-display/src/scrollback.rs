extern crate alloc;

use alloc::boxed::Box;
use core::ops::Range;

use bitfields::bitfields;
use common::constants::REGULAR_PAGE_SIZE;

use crate::{
    console::WIDTH, row_history::RowHistory, screen_char::ScreenChar,
    vga_memory::VgaMemory,
};

const SCROLLBACK_CELLS: usize =
    (64 * REGULAR_PAGE_SIZE) / size_of::<ScreenChar>();
const SCROLLBACK_ROWS: usize =
    (4 * REGULAR_PAGE_SIZE) / size_of::<RowSpan>();

const _: () =
    assert!(SCROLLBACK_CELLS <= 1 << 17, "`start` holds 17 bits");

/// Where one row's text sits in [`Scrollback::cells`], without its blank
/// ends.
#[bitfields]
pub struct RowSpan {
    pub start: B17,
    pub indent: B7,
    pub len: B8,
}

impl RowSpan {
    /// The columns of the screen row that hold this text.
    pub(crate) fn columns(&self) -> Range<usize> {
        let indent = self.get_indent() as usize;
        indent..indent + self.get_len() as usize
    }
}

/// Every finished row, stored without its blank ends.
///
/// Row numbers count from the oldest stored row. The live screen and the
/// view are two windows over these rows, and scrolling only moves the
/// view's window.
///
/// `cells` and `spans` are both rings, so a new row drops the oldest rows
/// when it doesn't fit.
pub struct Scrollback {
    cells: Box<[ScreenChar; SCROLLBACK_CELLS]>,
    spans: Box<[RowSpan; SCROLLBACK_ROWS]>,
    next_cell: usize,
    used_cells: usize,
    /// The slot in `spans` of the oldest row.
    oldest_slot: usize,
    row_count: usize,
    /// The row at the top of the live screen.
    screen_top: usize,
    /// How many rows the view sits above the live screen.
    view_offset: usize,
}

impl Default for Scrollback {
    fn default() -> Self {
        // SAFETY: all zeros is a valid `ScreenChar` and a valid `RowSpan`.
        let (cells, spans) = unsafe {
            (
                Box::new_zeroed().assume_init(),
                Box::new_zeroed().assume_init(),
            )
        };

        Self {
            cells,
            spans,
            next_cell: 0,
            used_cells: 0,
            oldest_slot: 0,
            row_count: 0,
            screen_top: 0,
            view_offset: 0,
        }
    }
}

impl Scrollback {
    /// Moves the view `lines` rows up, stopping at the oldest row.
    ///
    /// Leaving the live screen pushes the cursor's unfinished row, because
    /// drawing the view overwrites the only copy of it.
    pub(crate) fn scroll_view_up(
        &mut self,
        vga: &mut VgaMemory,
        cursor_row: usize,
        lines: usize,
    ) {
        if lines == 0 || !self.has_rows_above_screen() {
            return;
        }
        if !self.is_viewing_history() {
            self.push_row(&vga.rows()[cursor_row]);
        }
        self.view_offset =
            self.view_offset.saturating_add(lines).min(self.screen_top);
        self.draw_view(vga);
    }

    /// Moves the view `lines` rows down, stopping at the live screen.
    ///
    /// Reaching the live screen pops the row [`Self::scroll_view_up`]
    /// pushed, as the screen holds it again.
    pub(crate) fn scroll_view_down(
        &mut self,
        vga: &mut VgaMemory,
        lines: usize,
    ) {
        if !self.is_viewing_history() {
            return;
        }
        self.view_offset = self.view_offset.saturating_sub(lines);
        self.draw_view(vga);
        if !self.is_viewing_history() {
            self.pop_row();
        }
    }

    fn is_viewing_history(&self) -> bool { self.view_offset > 0 }

    /// Draws the rows from the top of the view, and blanks the screen past
    /// the newest row.
    fn draw_view(&self, vga: &mut VgaMemory) {
        let view_top = self.screen_top - self.view_offset;
        for (row, screen_row) in (view_top..).zip(vga.rows()) {
            if row < self.row_count {
                self.draw_row(row, screen_row);
            } else {
                screen_row.fill(ScreenChar::default());
            }
        }
    }

    /// Draws `row` with its blank ends back in place.
    fn draw_row(&self, row: usize, screen_row: &mut [ScreenChar; WIDTH]) {
        let span = self.spans[self.slot_of(row)];
        let start = span.get_start() as usize;
        let columns = span.columns();

        screen_row[..columns.start].fill(ScreenChar::default());
        for (offset, cell) in
            screen_row[columns.clone()].iter_mut().enumerate()
        {
            *cell = self.cells[(start + offset) % SCROLLBACK_CELLS];
        }
        screen_row[columns.end..].fill(ScreenChar::default());
    }

    /// The slot in `spans` of `row`.
    fn slot_of(&self, row: usize) -> usize {
        (self.oldest_slot + row) % SCROLLBACK_ROWS
    }

    /// Drops the oldest rows until `len` more cells and one more row fit.
    fn make_room(&mut self, len: usize) {
        while self.row_count == SCROLLBACK_ROWS
            || self.used_cells + len > SCROLLBACK_CELLS
        {
            let oldest = self.spans[self.oldest_slot];
            self.used_cells -= oldest.get_len() as usize;
            self.oldest_slot = (self.oldest_slot + 1) % SCROLLBACK_ROWS;
            self.row_count -= 1;
            self.screen_top -= 1;
        }
    }
}

impl RowHistory for Scrollback {
    /// Stores `row` as the newest row, dropping the oldest rows if it
    /// doesn't fit.
    fn push_row(&mut self, row: &[ScreenChar; WIDTH]) {
        let columns = text_columns(row);
        let text = &row[columns.clone()];

        self.make_room(text.len());

        for (offset, &cell) in text.iter().enumerate() {
            self.cells[(self.next_cell + offset) % SCROLLBACK_CELLS] =
                cell;
        }

        self.spans[self.slot_of(self.row_count)] = RowSpan::new()
            .start(self.next_cell as u32)
            .indent(columns.start as u8)
            .len(text.len() as u8);

        self.next_cell = (self.next_cell + text.len()) % SCROLLBACK_CELLS;
        self.used_cells += text.len();
        self.row_count += 1;
    }

    /// Removes the newest row, whose cells become free again.
    fn pop_row(&mut self) -> Option<RowSpan> {
        self.row_count -= 1;
        let span = self.spans[self.slot_of(self.row_count)];
        self.used_cells -= span.get_len() as usize;
        self.next_cell = span.get_start() as usize;
        Some(span)
    }

    fn advance_screen_top(&mut self) { self.screen_top += 1; }

    fn has_rows_above_screen(&self) -> bool { self.screen_top > 0 }

    fn restore_row_above_screen(
        &mut self,
        top_row: &mut [ScreenChar; WIDTH],
    ) {
        self.screen_top -= 1;
        self.draw_row(self.screen_top, top_row);
    }

    fn start_new_screen(&mut self) { self.screen_top = self.row_count; }

    fn leave_view(&mut self, vga: &mut VgaMemory) {
        self.scroll_view_down(vga, usize::MAX);
    }
}

/// The columns between `row`'s blank ends, which are `0..0` if the whole
/// row is blank.
fn text_columns(row: &[ScreenChar; WIDTH]) -> Range<usize> {
    let is_text = |cell: &ScreenChar| !cell.is_blank();
    let start = row.iter().position(is_text).unwrap_or(0);
    let end = row.iter().rposition(is_text).map_or(0, |last| last + 1);
    start..end
}
