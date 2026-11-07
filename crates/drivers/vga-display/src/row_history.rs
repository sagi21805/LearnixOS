use crate::{
    console::WIDTH, screen_char::ScreenChar, scrollback::RowSpan,
    vga_memory::VgaMemory,
};

/// What the console tells whoever keeps its finished rows.
pub(crate) trait RowHistory {
    /// Keeps `row` as the newest row.
    fn push_row(&mut self, row: &[ScreenChar; WIDTH]);
    /// Removes the newest row, and returns its span if one was kept.
    fn pop_row(&mut self) -> Option<RowSpan>;
    /// The live screen moved one row down the kept rows.
    fn advance_screen_top(&mut self);
    /// Whether a kept row sits above the live screen.
    fn has_rows_above_screen(&self) -> bool;
    /// Moves the live screen one row up the kept rows, and draws the row
    /// it reached onto `top_row`.
    fn restore_row_above_screen(
        &mut self,
        top_row: &mut [ScreenChar; WIDTH],
    );
    /// The live screen starts over, below every kept row.
    fn start_new_screen(&mut self);
    /// Draws the live screen again if a view of older rows is showing.
    fn leave_view(&mut self, vga: &mut VgaMemory);
}

/// Keeps nothing, for the console before the heap exists.
pub(crate) struct NoHistory;

impl RowHistory for NoHistory {
    fn push_row(&mut self, _row: &[ScreenChar; WIDTH]) {}

    fn pop_row(&mut self) -> Option<RowSpan> { None }

    fn advance_screen_top(&mut self) {}

    fn has_rows_above_screen(&self) -> bool { false }

    fn restore_row_above_screen(
        &mut self,
        _top_row: &mut [ScreenChar; WIDTH],
    ) {
    }

    fn start_new_screen(&mut self) {}

    fn leave_view(&mut self, _vga: &mut VgaMemory) {}
}
