use alloc::vec::Vec;
use core::{cmp::Ordering, mem, ops::Range};

use bitfields::bitfields;

use crate::{
    console::{CELLS, HEIGHT, WIDTH},
    row_history::RowHistory,
    screen_char::ScreenChar,
    vga_memory::VgaMemory,
};

/// The span of a row at the screen.
///
/// Indet, and len help to store less, if the row starts or ends in blanks.
#[bitfields]
pub struct RowSpan {
    /// The index in `cells` buffer of the row's first cell.
    pub start: B50,
    /// The index where texts start inside the row.
    pub indent: B7,
    /// How many cells the text takes.
    pub len: B7,
}

impl RowSpan {
    /// Finds the part of `row` worth keeping, for text that will sit at
    /// index `start` in `cells`. A blank row keeps nothing.
    fn from_row(start: usize, row: &[ScreenChar; WIDTH]) -> Self {
        let is_text = |cell: &ScreenChar| !cell.is_blank();
        let indent = row.iter().position(is_text).unwrap_or(0);
        let end = row.iter().rposition(is_text).map_or(0, |last| last + 1);
        RowSpan::new()
            .start(start as u64)
            .indent(indent as u8)
            .len((end - indent) as u8)
    }

    /// Where the text goes on a screen row the cursor after it.
    pub(crate) fn columns(&self) -> Range<usize> {
        let indent = self.get_indent() as usize;
        indent..indent + self.get_len() as usize
    }

    /// Where the text sits in `cells`
    fn cells(&self) -> Range<usize> {
        let start = self.get_start() as usize;
        start..start + self.get_len() as usize
    }
}

/// The kept rows, for after the heap exists.
#[derive(Default)]
pub struct Scrollback {
    /// The text of every kept row, one row after the other.
    cells: Vec<ScreenChar>,
    /// The spans of the rows.
    spans: Vec<RowSpan>,
    /// The `y` of the output area's first row.
    output_base: usize,
    /// How many rows the view is older than the output area.
    view_offset: usize,
}

impl Scrollback {
    /// Checks if the view is the latest.
    fn is_viewing_history(&self) -> bool { self.view_offset > 0 }

    /// The `y` of the view's first row.
    fn view_base(&self) -> usize { self.output_base - self.view_offset }

    /// Move the view to a certain view offset, and draw the screen with
    /// it.
    ///
    /// This function works for both directions.
    fn move_view_to(&mut self, vga: &mut VgaMemory, view_offset: usize) {
        let old_offset = mem::replace(&mut self.view_offset, view_offset);

        let fresh = match view_offset.cmp(&old_offset) {
            Ordering::Equal => return,
            Ordering::Greater => {
                let moved = (view_offset - old_offset).min(HEIGHT);
                vga.cells()
                    .copy_within(..CELLS - moved * WIDTH, moved * WIDTH);
                0..moved
            }
            Ordering::Less => {
                let moved = (old_offset - view_offset).min(HEIGHT);
                vga.cells().copy_within(moved * WIDTH.., 0);
                HEIGHT - moved..HEIGHT
            }
        };
        for (y, screen_row) in
            (self.view_base() + fresh.start..).zip(&mut vga.rows()[fresh])
        {
            self.draw_row(y, screen_row);
        }
    }

    /// Puts the kept row at `y` back on the screen row `at`.
    fn draw_row(&self, y: usize, at: &mut [ScreenChar; WIDTH]) {
        let span = self.spans[y];
        let columns = span.columns();

        at[..columns.start].fill(ScreenChar::default());
        at[columns.clone()].copy_from_slice(&self.cells[span.cells()]);
        at[columns.end..].fill(ScreenChar::default());
    }
}

impl RowHistory for Scrollback {
    fn push_row(&mut self, row: &[ScreenChar; WIDTH]) {
        let span = RowSpan::from_row(self.cells.len(), row);
        self.cells.extend_from_slice(&row[span.columns()]);
        self.spans.push(span);
    }

    fn pop_row(&mut self) -> Option<RowSpan> {
        let span = self.spans.pop()?;
        self.cells.truncate(span.cells().start);
        Some(span)
    }

    fn shift_output_up(&mut self, vga: &mut VgaMemory) {
        vga.shift_rows_up();
        self.output_base += 1;
    }

    fn shift_output_down(&mut self, vga: &mut VgaMemory) -> bool {
        let Some(prev) = self.output_base.checked_sub(1) else {
            return false;
        };
        vga.shift_rows_down();
        self.draw_row(prev, &mut vga.rows()[0]);
        self.output_base = prev;
        true
    }

    fn scroll_view_up(
        &mut self,
        vga: &mut VgaMemory,
        cursor_row: usize,
        lines: usize,
    ) {
        if lines == 0 || self.output_base == 0 {
            return;
        }
        if !self.is_viewing_history() {
            self.push_row(&vga.rows()[cursor_row]);
        }
        let view_offset =
            self.view_offset.saturating_add(lines).min(self.output_base);
        self.move_view_to(vga, view_offset);
    }

    fn scroll_view_down(&mut self, vga: &mut VgaMemory, lines: usize) {
        if !self.is_viewing_history() {
            return;
        }
        self.move_view_to(vga, self.view_offset.saturating_sub(lines));
        if !self.is_viewing_history() {
            self.pop_row();
        }
    }
}
