use core::{ops::RangeBounds, ptr, slice};

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

    /// The cells as screen rows of [`WIDTH`] cells.
    pub(crate) fn rows(&mut self) -> &mut [[ScreenChar; WIDTH]] {
        self.cells().as_chunks_mut().0
    }

    /// Blanks the given range of cells.
    pub(crate) fn blank_cells<R: RangeBounds<usize>>(&mut self, cells: R) {
        let range = slice::range(cells, ..CELLS);
        self.cells()[range].fill(ScreenChar::default());
    }

    /// Makes room for a new row at the bottom of the screen.
    pub(crate) fn shift_rows_up(&mut self) {
        self.cells().copy_within(WIDTH.., 0);
        self.rows()[HEIGHT - 1].fill(ScreenChar::default());
    }

    /// Makes room for an older row at the top of the screen, which the
    /// caller draws.
    pub(crate) fn shift_rows_down(&mut self) {
        self.cells().copy_within(..CELLS - WIDTH, WIDTH);
    }
}
