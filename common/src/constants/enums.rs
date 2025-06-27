use super::error::TableError;
use crate::constants::values::{
    BIG_PAGE_ALIGNMENT, BIG_PAGE_SIZE, HUGE_PAGE_ALIGNMENT, HUGE_PAGE_SIZE, REGULAR_PAGE_ALIGNMENT,
    REGULAR_PAGE_SIZE,
};
use core::{alloc::Layout, ptr::Alignment};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PageSize {
    /// 4Kib pages
    Regular = 0,
    /// 2Mib pages
    Big = 1,
    /// 1Gib pages
    Huge = 2,
}
#[derive(Clone)]
pub enum PageTableLevel {
    ForthLevel = 0,
    ThirdLevel = 1,
    SecondLevel = 2,
    FirstTable = 3,
}

impl PageTableLevel {
    /// Returns the next lower page table level, or `None` if already at the lowest level.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::PageTableLevel;
    /// assert_eq!(PageTableLevel::ThirdLevel.next(), Some(PageTableLevel::SecondLevel));
    /// assert_eq!(PageTableLevel::FirstTable.next(), None);
    /// ```
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::FirstTable => None,
            Self::SecondLevel => Some(Self::FirstTable),
            Self::ThirdLevel => Some(Self::SecondLevel),
            Self::ForthLevel => Some(Self::ThirdLevel),
        }
    }
    /// Returns the previous higher page table level, or an error if already at the highest level.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: The previous higher `PageTableLevel`.
    /// - `Err(TableError::Full)`: If called on `ForthLevel`, indicating no higher level exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::PageTableLevel;
    ///
    /// let level = PageTableLevel::SecondLevel;
    /// assert_eq!(level.prev().unwrap(), PageTableLevel::ThirdLevel);
    ///
    /// let top = PageTableLevel::ForthLevel;
    /// assert!(top.prev().is_err());
    /// ```
    pub fn prev(&self) -> Result<Self, TableError> {
        match self {
            Self::FirstTable => Ok(Self::SecondLevel),
            Self::SecondLevel => Ok(Self::ThirdLevel),
            Self::ThirdLevel => Ok(Self::ForthLevel),
            Self::ForthLevel => Err(TableError::Full),
        }
    }
    /// Returns the numeric representation of the page table level as a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// let level = PageTableLevel::SecondLevel;
    /// assert_eq!(level.as_usize(), 2);
    /// ```
    pub fn as_usize(&self) -> usize {
        self.clone() as usize
    }
}

impl PageSize {
    /// Returns the memory alignment associated with the page size.
    ///
    /// # Examples
    ///
    /// ```
    /// let alignment = PageSize::Regular.alignment();
    /// assert_eq!(alignment, REGULAR_PAGE_ALIGNMENT);
    /// ```
    pub fn alignment(&self) -> Alignment {
        match self {
            PageSize::Regular => REGULAR_PAGE_ALIGNMENT,

            PageSize::Big => BIG_PAGE_ALIGNMENT,

            PageSize::Huge => HUGE_PAGE_ALIGNMENT,
        }
    }

    /// Determines if the page size is equal to or larger than the specified page table level.
    ///
    /// Returns `true` if the page size corresponds to a level that is equal to or exceeds the given `PageTableLevel`.
    ///
    /// # Examples
    ///
    /// ```
    /// let page_size = PageSize::Big;
    /// let level = PageTableLevel::SecondLevel;
    /// assert!(page_size.exceeds(&level));
    /// ```
    pub fn exceeds(&self, table_level: &PageTableLevel) -> bool {
        return (3 - self.clone() as usize) <= table_level.as_usize();
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

    /// Returns the `PageSize` corresponding to the given memory alignment, or `None` if the alignment does not match a known page size.
    ///
    /// # Examples
    ///
    /// ```
    /// let page_size = PageSize::from_alignment(REGULAR_PAGE_ALIGNMENT);
    /// assert_eq!(page_size, Some(PageSize::Regular));
    /// ```
    pub const fn from_alignment(alignment: Alignment) -> Option<Self> {
        match alignment {
            REGULAR_PAGE_ALIGNMENT => Some(Self::Regular),
            BIG_PAGE_ALIGNMENT => Some(Self::Big),
            HUGE_PAGE_ALIGNMENT => Some(Self::Huge),
            _ => None,
        }
    }

    /// Returns the number of 4KiB regular pages contained in this page size.
    ///
    /// For example, a `Big` page (2MiB) contains 512 regular pages, and a `Huge` page (1GiB) contains 262,144 regular pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::PageSize;
    ///
    /// assert_eq!(PageSize::Regular.size_in_regular_pages(), 1);
    /// assert_eq!(PageSize::Big.size_in_regular_pages(), 512);
    /// assert_eq!(PageSize::Huge.size_in_regular_pages(), 262_144);
    /// ```
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
