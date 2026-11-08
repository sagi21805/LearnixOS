use alloc::vec::Vec;
use core::{cmp::Ordering, ops::Range};

use bitfields::bitfields;

use crate::{
    console::{CELLS, HEIGHT, WIDTH},
    row_history::RowHistory,
    screen_char::ScreenChar,
    vga_memory::VgaMemory,
};

/// Where one row's text sits in [`Scrollback::cells`], without its blank
/// ends.
#[bitfields]
pub struct RowSpan {
    /// The index in `cells` of the text's first cell.
    pub start: B50,
    /// The `x` of the text's first cell, which is how many blanks the row
    /// had on its left.
    pub indent: B7,
    /// How many cells the text takes.
    pub len: B7,
}

impl RowSpan {
    /// The span of `row`'s text between its blank ends, with the text
    /// stored from index `start` in `cells`. A blank row gets an empty
    /// span at `x` 0.
    fn from_row(start: usize, row: &[ScreenChar; WIDTH]) -> Self {
        let is_text = |cell: &ScreenChar| !cell.is_blank();
        let indent = row.iter().position(is_text).unwrap_or(0);
        let end = row.iter().rposition(is_text).map_or(0, |last| last + 1);
        RowSpan::new()
            .start(start as u64)
            .indent(indent as u8)
            .len((end - indent) as u8)
    }

    /// The `x` range of the screen row that holds this text.
    pub(crate) fn columns(&self) -> Range<usize> {
        let indent = self.get_indent() as usize;
        indent..indent + self.get_len() as usize
    }

    /// The indices in `cells` that hold this text.
    fn cells(&self) -> Range<usize> {
        let start = self.get_start() as usize;
        start..start + self.get_len() as usize
    }
}

/// Every finished row, stored without its blank ends.
#[derive(Default)]
pub struct Scrollback {
    /// The text of every row, one row after the other.
    cells: Vec<ScreenChar>,
    spans: Vec<RowSpan>,
    /// The `y` of the output area's first row.
    base_y: usize,
    /// How many rows the view's first row sits above `base_y`.
    view_offset: usize,
}

impl Scrollback {
    fn is_viewing_history(&self) -> bool { self.view_offset > 0 }

    /// The `y` of the view's first row.
    fn view_y(&self) -> usize { self.base_y - self.view_offset }

    /// Moves the view to start `view_offset` rows above `base_y`, by
    /// shifting the rows that stay in view and drawing only the rows that
    /// come into it.
    ///
    /// The shift relies on VGA memory showing the view before the move.
    /// Every drawn row is kept, because the view only leaves the output
    /// area while `base_y > 0`, which keeps the cursor on the bottom
    /// screen row, and `scroll_view_up` pushes that row too.
    fn move_view_to(&mut self, vga: &mut VgaMemory, view_offset: usize) {
        let from = self.view_y();
        self.view_offset = view_offset;
        let to = self.view_y();

        let fresh = match to.cmp(&from) {
            Ordering::Equal => return,
            Ordering::Less => {
                let moved = (from - to).min(HEIGHT);
                vga.cells()
                    .copy_within(..CELLS - moved * WIDTH, moved * WIDTH);
                0..moved
            }
            Ordering::Greater => {
                let moved = (to - from).min(HEIGHT);
                vga.cells().copy_within(moved * WIDTH.., 0);
                HEIGHT - moved..HEIGHT
            }
        };
        for (y, screen_row) in
            (to + fresh.start..).zip(&mut vga.rows()[fresh])
        {
            self.draw_row(y, screen_row);
        }
    }

    /// Draws the row at `y` with its blank ends back in place.
    fn draw_row(&self, y: usize, screen_row: &mut [ScreenChar; WIDTH]) {
        let span = self.spans[y];
        let columns = span.columns();

        screen_row[..columns.start].fill(ScreenChar::default());
        screen_row[columns.clone()]
            .copy_from_slice(&self.cells[span.cells()]);
        screen_row[columns.end..].fill(ScreenChar::default());
    }
}

impl RowHistory for Scrollback {
    /// Stores `row` as the newest row.
    fn push_row(&mut self, row: &[ScreenChar; WIDTH]) {
        let span = RowSpan::from_row(self.cells.len(), row);
        self.cells.extend_from_slice(&row[span.columns()]);
        self.spans.push(span);
    }

    /// Removes the newest row, along with its cells.
    fn pop_row(&mut self) -> Option<RowSpan> {
        let span = self.spans.pop()?;
        self.cells.truncate(span.cells().start);
        Some(span)
    }

    fn advance_base_y(&mut self) { self.base_y += 1; }

    fn scroll_output_down(&mut self, vga: &mut VgaMemory) -> bool {
        let Some(y_above) = self.base_y.checked_sub(1) else {
            return false;
        };
        vga.cells().copy_within(..CELLS - WIDTH, WIDTH);
        self.draw_row(y_above, &mut vga.rows()[0]);
        self.base_y = y_above;
        true
    }

    /// Moves the view `lines` rows toward older rows, stopping at `y` 0.
    ///
    /// Leaving the output area pushes the cursor's unfinished row, because
    /// drawing the view overwrites the only copy of it.
    fn scroll_view_up(
        &mut self,
        vga: &mut VgaMemory,
        cursor_row: usize,
        lines: usize,
    ) {
        if lines == 0 || self.base_y == 0 {
            return;
        }
        if !self.is_viewing_history() {
            self.push_row(&vga.rows()[cursor_row]);
        }
        let view_offset =
            self.view_offset.saturating_add(lines).min(self.base_y);
        self.move_view_to(vga, view_offset);
    }

    /// Moves the view `lines` rows toward newer rows, stopping at
    /// `base_y`.
    ///
    /// Reaching the output area pops the row `scroll_view_up` pushed, as
    /// VGA memory holds it again.
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
