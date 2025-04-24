use core::{alloc::Layout, ptr::Alignment};

use crate::values::{
    BIG_PAGE_ALIGNMENT, BIG_PAGE_SIZE, HUGE_PAGE_ALIGNMENT, HUGE_PAGE_SIZE, REGULAR_PAGE_ALIGNMENT,
    REGULAR_PAGE_SIZE,
};

pub enum Interrupts {
    VIDEO = 0x10,
    DISK = 0x13,
}

pub enum Sections {
    Null = 0x0,
    KernelCode = 0x8,
    KernelData = 0x10,
}

pub enum Disk {
    ExtendedRead = 0x42,
}

pub enum Video {
    SetMode = 0x0,
}
#[allow(non_camel_case_types)]
pub enum VideoModes {
    /// VGA Common Text Mode ->
    ///
    /// Text resolution 80x25
    ///
    /// PixelBox resolution 9x16
    ///
    /// Pixel resolution 720x400
    VGA_TX_80X25_PB_9X16_PR_720X400 = 0x3,
}

pub enum PacketSize {
    Default = 0x10,
}

#[derive(Clone)]
pub enum PageSize {
    /// 4Kib pages
    Regular = 1,
    /// 2Mib pages
    Big = 2,
    /// 1Gib pages
    Huge = 3,
}

impl PageSize {
    pub fn alignment(&self) -> Alignment {
        match self {
            PageSize::Regular => REGULAR_PAGE_ALIGNMENT,

            PageSize::Big => BIG_PAGE_ALIGNMENT,

            PageSize::Huge => HUGE_PAGE_ALIGNMENT,
        }
    }

    /// Determines the appropriate `PageSizeAlignment` for a given memory layout.
    ///
    /// # Parameters
    ///
    /// - `layout`: A [`Layout`] struct containing the memory size and alignment.
    pub const fn from_layout(layout: Layout) -> Option<PageSize> {
        match (layout.size(), layout.align()) {
            (REGULAR_PAGE_SIZE, val) if val == REGULAR_PAGE_ALIGNMENT.as_usize() => {
                Some(PageSize::Regular)
            }
            (BIG_PAGE_SIZE, val) if val == BIG_PAGE_ALIGNMENT.as_usize() => Some(PageSize::Big),
            (HUGE_PAGE_SIZE, val) if val == HUGE_PAGE_ALIGNMENT.as_usize() => Some(PageSize::Huge),

            _ => None,
        }
    }

    pub const fn size_in_pages(&self) -> usize {
        match self {
            PageSize::Regular => 1,

            PageSize::Big => 512,

            PageSize::Huge => 512 * 512,
        }
    }
}
