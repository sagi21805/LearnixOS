use core::ptr::Alignment;

pub const PAGE_DIRECTORY_ENTRIES: usize = 512;
pub const REGULAR_PAGE_SIZE: usize = 4096;
pub const BIG_PAGE_SIZE: usize = REGULAR_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;
pub const HUGE_PAGE_SIZE: usize = BIG_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;
#[allow(non_upper_case_globals)]
pub const KiB: usize = 1024;
#[allow(non_upper_case_globals)]
pub const MiB: usize = 1024 * 1024;
pub const REGULAR_PAGE_ALIGNMENT: Alignment =
    unsafe { Alignment::new_unchecked(REGULAR_PAGE_SIZE) };
pub const BIG_PAGE_ALIGNMENT: Alignment = unsafe { Alignment::new_unchecked(BIG_PAGE_SIZE) };
pub const HUGE_PAGE_ALIGNMENT: Alignment = unsafe { Alignment::new_unchecked(HUGE_PAGE_SIZE) };
pub const MEMORY_MAP_MAGIC_NUMBER: u32 = unsafe { core::mem::transmute([b'P', b'A', b'M', b'S']) }; // 'SMAP' in little endian
