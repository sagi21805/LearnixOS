use core::ptr::Alignment;

pub const PAGE_DIRECTORY_ENTRIES: usize = 512;
pub const REGULAR_PAGE_SIZE: usize = 4096;
pub const BIG_PAGE_SIZE: usize = REGULAR_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;
pub const HUGE_PAGE_SIZE: usize = BIG_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;

pub const REGULAR_PAGE_ALIGNMENT: Alignment = unsafe { Alignment::new_unchecked(REGULAR_PAGE_SIZE) };
pub const BIG_PAGE_ALIGNMENT: Alignment = unsafe { Alignment::new_unchecked(BIG_PAGE_SIZE) };
pub const HUGE_PAGE_ALIGNMENT: Alignment = unsafe { Alignment::new_unchecked(HUGE_PAGE_SIZE) };