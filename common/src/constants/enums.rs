use core::{alloc::Layout, ptr::Alignment};

use crate::constants::values::{
    BIG_PAGE_ALIGNMENT, BIG_PAGE_SIZE, HUGE_PAGE_ALIGNMENT, HUGE_PAGE_SIZE, REGULAR_PAGE_ALIGNMENT,
    REGULAR_PAGE_SIZE,
};

pub enum Interrupts {
    VIDEO = 0x10,
    DISK = 0x13,
    MEMORY = 0x15,
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

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MemoryRegionType {
    Unknown = 0u32,
    Usable = 0x1u32,
    Reserved = 0x2u32,
    Reclaimable = 0x3u32,
    ACPINVS = 0x4u32,
    BadMemory = 0x5u32,
}

pub enum PacketSize {
    Default = 0x10,
}

#[derive(Clone, Debug)]
pub enum PageSize {
    /// 4Kib pages
    Regular = 0,
    /// 2Mib pages
    Big = 1,
    /// 1Gib pages
    Huge = 2,
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
    pub const fn from_layout(layout: Layout) -> Option<Self> {
        match layout.align() {
            val if val == REGULAR_PAGE_ALIGNMENT.as_usize() => Some(PageSize::Regular),
            val if val == BIG_PAGE_ALIGNMENT.as_usize() => Some(PageSize::Big),
            val if val == HUGE_PAGE_ALIGNMENT.as_usize() => Some(PageSize::Huge),

            _ => None,
        }
    }

    pub const fn from_alignment(alignment: Alignment) -> Option<Self> {
        match alignment {
            REGULAR_PAGE_ALIGNMENT => Some(Self::Regular),
            BIG_PAGE_ALIGNMENT => Some(Self::Big),
            HUGE_PAGE_ALIGNMENT => Some(Self::Huge),
            _ => None,
        }
    }

    pub const fn size_in_regular_pages(&self) -> usize {
        match self {
            PageSize::Regular => 1,

            PageSize::Big => 512,

            PageSize::Huge => 512 * 512,
        }
    }
}

impl Into<Layout> for PageSize {
    fn into(self) -> Layout {
        unsafe {
            match self {
                Self::Regular => Layout::from_size_align_unchecked(
                    REGULAR_PAGE_SIZE,
                    REGULAR_PAGE_ALIGNMENT.as_usize(),
                ),
                Self::Big => {
                    Layout::from_size_align_unchecked(BIG_PAGE_SIZE, BIG_PAGE_ALIGNMENT.as_usize())
                }
                Self::Huge => Layout::from_size_align_unchecked(
                    HUGE_PAGE_SIZE,
                    HUGE_PAGE_ALIGNMENT.as_usize(),
                ),
            }
        }
    }
}
