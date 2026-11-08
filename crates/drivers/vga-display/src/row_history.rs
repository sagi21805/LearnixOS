use core::ptr::NonNull;

use crate::{
    console::WIDTH, screen_char::ScreenChar, scrollback::RowSpan,
    vga_memory::VgaMemory,
};

/// What the console tells whoever keeps its finished rows.
pub(crate) trait RowHistory: Send {
    /// Keeps `row` as the newest row.
    fn push_row(&mut self, row: &[ScreenChar; WIDTH]);
    /// Removes the newest row, and returns its span if one was kept.
    fn pop_row(&mut self) -> Option<RowSpan>;
    /// The output area's first row moved one row toward newer rows.
    fn advance_base_y(&mut self);
    /// Moves the output area one row toward older rows and draws the kept
    /// row at its new top, or returns `false` if no kept row is above it.
    fn scroll_output_down(&mut self, vga: &mut VgaMemory) -> bool;
    /// Moves the view `lines` rows up, toward older rows.
    fn scroll_view_up(
        &mut self,
        vga: &mut VgaMemory,
        cursor_row: usize,
        lines: usize,
    );
    /// Moves the view `lines` rows down, toward the output area.
    fn scroll_view_down(&mut self, vga: &mut VgaMemory, lines: usize);
}

/// Keeps nothing, for the console before the heap exists.
pub(crate) struct NoHistory;

impl NoHistory {
    /// A `&'static mut NoHistory`, built in a `const`.
    pub(crate) const fn leaked() -> &'static mut NoHistory {
        // SAFETY: `NoHistory` is zero-sized, so a dangling, aligned
        // pointer is a valid reference to it that aliases nothing.
        unsafe { NonNull::dangling().as_mut() }
    }
}

impl RowHistory for NoHistory {
    fn push_row(&mut self, _row: &[ScreenChar; WIDTH]) {}

    fn pop_row(&mut self) -> Option<RowSpan> { None }

    fn advance_base_y(&mut self) {}

    fn scroll_output_down(&mut self, _vga: &mut VgaMemory) -> bool {
        false
    }

    fn scroll_view_up(
        &mut self,
        _vga: &mut VgaMemory,
        _cursor_row: usize,
        _lines: usize,
    ) {
    }

    fn scroll_view_down(&mut self, _vga: &mut VgaMemory, _lines: usize) {}
}
