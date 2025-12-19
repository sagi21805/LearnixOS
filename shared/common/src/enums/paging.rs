use core::{alloc::Layout, ptr::Alignment};
use num_enum::TryFromPrimitive;
use strum_macros::{EnumIter, VariantArray};

use crate::{
    constants::{
        BIG_PAGE_ALIGNMENT, BIG_PAGE_SIZE, HUGE_PAGE_ALIGNMENT,
        HUGE_PAGE_SIZE, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    error::{ConversionError, TableError},
};

#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    EnumIter,
    TryFromPrimitive,
    VariantArray,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum PageTableLevel {
    PML4 = 4,
    PDPT = 3,
    PD = 2,
    PT = 1,
}

impl PageTableLevel {
    pub fn next(&self) -> Option<Self> {
        let n = (*self as u8) - 1;
        (n > 0).then(|| unsafe { core::mem::transmute(n) })
    }

    pub fn prev(&self) -> Result<Self, TableError> {
        let n = (*self as u8) + 1;
        (n <= 4)
            .then(|| unsafe { core::mem::transmute(n) })
            .ok_or(TableError::Full)
    }
}
#[repr(u8)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, EnumIter, TryFromPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum PageSize {
    /// 4Kib pages
    Regular = 2,
    /// 2Mib pages
    Big = 1,
    /// 1Gib pages
    Huge = 0,
}

impl PageSize {
    pub const fn alignment(&self) -> Alignment {
        match self {
            PageSize::Regular => REGULAR_PAGE_ALIGNMENT,

            PageSize::Big => BIG_PAGE_ALIGNMENT,

            PageSize::Huge => HUGE_PAGE_ALIGNMENT,
        }
    }

    pub fn exceeds(&self, table_level: PageTableLevel) -> bool {
        (3 - *self as usize) <= table_level as usize
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

impl const From<PageSize> for Layout {
    fn from(val: PageSize) -> Self {
        unsafe {
            match val {
                PageSize::Regular => Layout::from_size_align_unchecked(
                    REGULAR_PAGE_SIZE,
                    REGULAR_PAGE_ALIGNMENT.as_usize(),
                ),
                PageSize::Big => Layout::from_size_align_unchecked(
                    BIG_PAGE_SIZE,
                    BIG_PAGE_ALIGNMENT.as_usize(),
                ),
                PageSize::Huge => Layout::from_size_align_unchecked(
                    HUGE_PAGE_SIZE,
                    HUGE_PAGE_ALIGNMENT.as_usize(),
                ),
            }
        }
    }
}
