use core::ptr;

use common::constants::VGA_BUFFER_PTR;

use crate::{
    console::{CELLS, HEIGHT, WIDTH},
    screen_char::ScreenChar,
};

/// The only path to VGA text memory.
///
/// Two live `&mut` to the same bytes are UB even on one core, so every
/// reference to the cells borrows from this handle. As its own field of
/// `Screen`, it also lets the screen lend the cells to its history while
/// the history is borrowed.
pub(crate) struct VgaMemory(());

impl VgaMemory {
    /// The handle, which `SCREEN` creates once.
    ///
    /// # Safety
    ///
    /// Only one `VgaMemory` may exist, because [`Self::cells`] gives out
    /// exclusive access to the same memory based on it.
    pub(crate) const unsafe fn new() -> Self { Self(()) }

    /// Every cell of the screen, row after row.
    pub(crate) fn cells(&mut self) -> &mut [ScreenChar; CELLS] {
        let cells =
            ptr::with_exposed_provenance_mut(VGA_BUFFER_PTR as usize);
        // SAFETY: the boot page tables map VGA text memory, any byte is a
        // valid `ScreenChar`, and `&mut self` borrows the only handle.
        unsafe { &mut *cells }
    }

    /// The cells as rows of [`WIDTH`].
    pub(crate) fn rows(&mut self) -> &mut [[ScreenChar; WIDTH]] {
        self.cells().as_chunks_mut().0
    }

    /// Moves every row up by one and blanks the last row.
    pub(crate) fn shift_rows_up(&mut self) {
        self.cells().copy_within(WIDTH.., 0);
        self.rows()[HEIGHT - 1].fill(ScreenChar::default());
    }

    /// Moves every row down by one, leaving the first row as it was.
    pub(crate) fn shift_rows_down(&mut self) {
        self.cells().copy_within(..CELLS - WIDTH, WIDTH);
    }
}
