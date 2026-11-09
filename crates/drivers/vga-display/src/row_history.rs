use core::ptr::NonNull;

use crate::{
    console::WIDTH, screen_char::ScreenChar, scrollback::RowSpan,
    vga_memory::VgaMemory,
};

/// Keeps the rows the console finishes, so text that left the screen can
/// be read again and backspace can reopen a finished row.
pub(crate) trait RowHistory: Send {
    /// Saves a row the cursor just finished.
    fn push_row(&mut self, row: &[ScreenChar; WIDTH]);
    /// Hands the newest kept row back
    fn pop_row(&mut self) -> Option<RowSpan>;
    /// Makes room for a new row once the cursor runs past the last cell.
    fn shift_output_up(&mut self, vga: &mut VgaMemory);
    /// Bring back the kept row before the output area. Returns `false` if
    /// there is none.
    fn shift_output_down(&mut self, vga: &mut VgaMemory) -> bool;
    /// Shows older text, for reading what left the screen. Writing isn't
    /// affected, as the output area and the cursor stay where they are.
    fn scroll_view_up(
        &mut self,
        vga: &mut VgaMemory,
        cursor_row: usize,
        lines: usize,
    );
    /// Shows newer text, on the way back from reading history.
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

    fn shift_output_up(&mut self, vga: &mut VgaMemory) {
        vga.shift_rows_up();
    }

    fn shift_output_down(&mut self, _vga: &mut VgaMemory) -> bool { false }

    fn scroll_view_up(
        &mut self,
        _vga: &mut VgaMemory,
        _cursor_row: usize,
        _lines: usize,
    ) {
    }

    fn scroll_view_down(&mut self, _vga: &mut VgaMemory, _lines: usize) {}
}
