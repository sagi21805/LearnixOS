extern crate alloc;

use alloc::boxed::Box;
use core::array;

use bitfields::bitfields;
use common::constants::REGULAR_PAGE_SIZE;

use crate::{
    console::{HEIGHT, WIDTH},
    screen_char::ScreenChar,
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

/// Rows that left the screen, stored without their blank ends.
///
/// `cells` and `row_table` are both rings, so a new row drops the oldest
/// rows when it doesn't fit. Most rows are short, so this holds more
/// history than storing every row at full width, and row `n` is still one
/// lookup away.
pub struct Scrollback {
    cells: Box<[ScreenChar; SCROLLBACK_CELLS]>,
    row_table: Box<[RowSpan; SCROLLBACK_ROWS]>,
    next_cell: usize,
    used_cells: usize,
    oldest_row: usize,
    row_count: usize,
    /// Rows between the bottom of the view and the newest row. Anything
    /// but 0 means the newest [`HEIGHT`] rows are a copy of the screen.
    view_offset: usize,
}

impl Default for Scrollback {
    fn default() -> Self {
        // SAFETY: all zeros is a valid `ScreenChar` and a valid `RowSpan`.
        let (cells, row_table) = unsafe {
            (
                Box::new_zeroed().assume_init(),
                Box::new_zeroed().assume_init(),
            )
        };

        Self {
            cells,
            row_table,
            next_cell: 0,
            used_cells: 0,
            oldest_row: 0,
            row_count: 0,
            view_offset: 0,
        }
    }
}

impl Scrollback {
    pub(crate) fn is_viewing_history(&self) -> bool {
        self.view_offset > 0
    }

    pub(crate) fn has_history(&self) -> bool { self.row_count > 0 }

    /// Moves the view towards older rows, stopping at the oldest.
    pub(crate) fn view_older(&mut self, lines: usize) {
        let oldest_view = self.row_count.saturating_sub(HEIGHT);
        self.view_offset =
            self.view_offset.saturating_add(lines).min(oldest_view);
    }

    /// Moves the view towards newer rows, stopping at the newest.
    pub(crate) fn view_newer(&mut self, lines: usize) {
        self.view_offset = self.view_offset.saturating_sub(lines);
    }

    /// Stores `row` without its blank ends, dropping the oldest rows if it
    /// doesn't fit.
    pub(crate) fn push_row(&mut self, row: &[ScreenChar; WIDTH]) {
        let indent = row
            .iter()
            .position(|cell| !cell.is_blank())
            .unwrap_or(WIDTH);
        let end = row
            .iter()
            .rposition(|cell| !cell.is_blank())
            .map_or(indent, |last| last + 1);
        let text = &row[indent..end];

        self.make_room(text.len());

        for (offset, &cell) in text.iter().enumerate() {
            self.cells[(self.next_cell + offset) % SCROLLBACK_CELLS] =
                cell;
        }

        let newest = (self.oldest_row + self.row_count) % SCROLLBACK_ROWS;
        self.row_table[newest] = RowSpan::new()
            .start(self.next_cell as u32)
            .indent(indent as u8)
            .len(text.len() as u8);

        self.next_cell = (self.next_cell + text.len()) % SCROLLBACK_CELLS;
        self.used_cells += text.len();
        self.row_count += 1;
    }

    /// Removes the `count` newest rows, whose cells become free again.
    pub(crate) fn forget_newest_rows(&mut self, count: usize) {
        for _ in 0..count {
            self.row_count -= 1;
            let span = self.row_table
                [(self.oldest_row + self.row_count) % SCROLLBACK_ROWS];
            self.used_cells -= span.get_len() as usize;
            self.next_cell = span.get_start() as usize;
        }
    }

    /// The row at `screen_row` of the current view, with its blank ends
    /// back in place.
    pub(crate) fn visible_row(
        &self,
        screen_row: usize,
    ) -> [ScreenChar; WIDTH] {
        let view_start = self.row_count - HEIGHT - self.view_offset;
        let span =
            self.row_table[(self.oldest_row + view_start + screen_row)
                % SCROLLBACK_ROWS];
        let start = span.get_start() as usize;
        let indent = span.get_indent() as usize;
        let len = span.get_len() as usize;

        array::from_fn(|column| match column.checked_sub(indent) {
            Some(offset) if offset < len => {
                self.cells[(start + offset) % SCROLLBACK_CELLS]
            }
            _ => ScreenChar::default(),
        })
    }

    /// Drops the oldest rows until `len` more cells and one more row fit.
    fn make_room(&mut self, len: usize) {
        while self.row_count == SCROLLBACK_ROWS
            || self.used_cells + len > SCROLLBACK_CELLS
        {
            let oldest = self.row_table[self.oldest_row];
            self.used_cells -= oldest.get_len() as usize;
            self.oldest_row = (self.oldest_row + 1) % SCROLLBACK_ROWS;
            self.row_count -= 1;
        }
    }
}
