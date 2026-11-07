use core::ptr;

use common::constants::VGA_BUFFER_PTR;

use crate::{
    console::{CELLS, WIDTH},
    screen_char::ScreenChar,
};

/// The only handle to VGA text memory.
///
/// It isn't `Clone` or `Copy`, and its field is private, so getting a
/// second one takes `unsafe`.
pub(crate) struct VgaMemory(());

impl VgaMemory {
    /// # Safety
    ///
    /// Only one `VgaMemory` may exist, because [`Self::cells`] gives out
    /// exclusive access to the same memory based on it.
    pub(crate) const unsafe fn new() -> Self { Self(()) }

    pub(crate) fn cells(&mut self) -> &mut [ScreenChar; CELLS] {
        let cells =
            ptr::with_exposed_provenance_mut(VGA_BUFFER_PTR as usize);
        // SAFETY: the boot page tables map VGA text memory, any byte is a
        // valid `ScreenChar`, and `&mut self` borrows the only handle.
        unsafe { &mut *cells }
    }

    pub(crate) fn rows(&mut self) -> &mut [[ScreenChar; WIDTH]] {
        self.cells().as_chunks_mut().0
    }
}
