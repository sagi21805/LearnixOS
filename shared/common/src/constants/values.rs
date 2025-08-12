use core::ptr::Alignment;

use crate::enums::paging;

#[cfg(feature = "paging")]
pub const PAGE_DIRECTORY_ENTRIES: usize = 512;
#[cfg(feature = "paging")]
pub const REGULAR_PAGE_SIZE: usize = 4096;
#[cfg(feature = "paging")]
pub const BIG_PAGE_SIZE: usize = REGULAR_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;
#[cfg(feature = "paging")]
pub const HUGE_PAGE_SIZE: usize = BIG_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;
#[cfg(feature = "kernel")]
#[allow(non_upper_case_globals)]
pub const KiB: usize = 1024;
#[cfg(feature = "kernel")]
#[allow(non_upper_case_globals)]
#[cfg(feature = "kernel")]
pub const MiB: usize = 1024 * 1024;
#[cfg(feature = "paging")]
pub const REGULAR_PAGE_ALIGNMENT: Alignment =
    unsafe { Alignment::new_unchecked(REGULAR_PAGE_SIZE) };
#[cfg(feature = "paging")]
pub const BIG_PAGE_ALIGNMENT: Alignment = unsafe { Alignment::new_unchecked(BIG_PAGE_SIZE) };
#[cfg(feature = "paging")]
pub const HUGE_PAGE_ALIGNMENT: Alignment = unsafe { Alignment::new_unchecked(HUGE_PAGE_SIZE) };
#[cfg(feature = "first_stage")]
pub const MEMORY_MAP_MAGIC_NUMBER: u32 = u32::from_le_bytes([b'P', b'A', b'M', b'S']); // 'SMAP' in little endian
#[cfg(feature = "paging")]
pub const ENTRY_ADDRESS_MASK: u64 = 0x0000_fffffffff_000;
