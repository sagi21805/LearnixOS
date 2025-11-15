use core::{alloc::Layout, ptr::Alignment};

use crate::{
    constants::{
        BIG_PAGE_ALIGNMENT, BIG_PAGE_SIZE, HUGE_PAGE_ALIGNMENT,
        HUGE_PAGE_SIZE, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    error::TableError,
};
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum PageSize {
    /// 4Kib pages
    Regular = 0,
    /// 2Mib pages
    Big = 1,
    /// 1Gib pages
    Huge = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageTableLevel {
    ForthLevel = 0,
    ThirdLevel = 1,
    SecondLevel = 2,
    FirstTable = 3,
}

impl PageTableLevel {
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::FirstTable => None,
            Self::SecondLevel => Some(Self::FirstTable),
            Self::ThirdLevel => Some(Self::SecondLevel),
            Self::ForthLevel => Some(Self::ThirdLevel),
        }
    }
    pub fn prev(&self) -> Result<Self, TableError> {
        match self {
            Self::FirstTable => Ok(Self::SecondLevel),
            Self::SecondLevel => Ok(Self::ThirdLevel),
            Self::ThirdLevel => Ok(Self::ForthLevel),
            Self::ForthLevel => Err(TableError::Full),
        }
    }

    pub fn as_usize(&self) -> usize {
        self.clone() as usize
    }
}

impl PageSize {
    pub fn alignment(&self) -> Alignment {
        match self {
            PageSize::Regular => REGULAR_PAGE_ALIGNMENT,

            PageSize::Big => BIG_PAGE_ALIGNMENT,

            PageSize::Huge => HUGE_PAGE_ALIGNMENT,
        }
    }

    pub fn exceeds(&self, table_level: PageTableLevel) -> bool {
        return (3 - self.clone() as usize) <= table_level.as_usize();
    }

    /// Determines the appropriate `PageSizeAlignment` for a
    /// given memory layout.
    ///
    /// # Parameters
    ///
    /// - `layout`: A [`Layout`] struct containing the memory size and
    ///   alignment.
    pub const fn from_layout(layout: Layout) -> Option<Self> {
        match layout.align() {
            val if val == REGULAR_PAGE_ALIGNMENT.as_usize() => {
                Some(PageSize::Regular)
            }
            val if val == BIG_PAGE_ALIGNMENT.as_usize() => {
                Some(PageSize::Big)
            }
            val if val == HUGE_PAGE_ALIGNMENT.as_usize() => {
                Some(PageSize::Huge)
            }

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
                Self::Big => Layout::from_size_align_unchecked(
                    BIG_PAGE_SIZE,
                    BIG_PAGE_ALIGNMENT.as_usize(),
                ),
                Self::Huge => Layout::from_size_align_unchecked(
                    HUGE_PAGE_SIZE,
                    HUGE_PAGE_ALIGNMENT.as_usize(),
                ),
            }
        }
    }
}
