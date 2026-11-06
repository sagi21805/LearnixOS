use core::mem::Alignment;
pub const PAGE_DIRECTORY_ENTRIES: usize = 512;
pub const REGULAR_PAGE_SIZE: usize = 4096;
pub const BIG_PAGE_SIZE: usize =
    REGULAR_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;
pub const HUGE_PAGE_SIZE: usize = BIG_PAGE_SIZE * PAGE_DIRECTORY_ENTRIES;
/// The GiBs, from address 0, that the boot map identity-maps.
pub const BOOT_MAPPED_GIBS: usize = 1;
#[allow(non_upper_case_globals)]
pub const KiB: usize = 1024;
#[allow(non_upper_case_globals)]
pub const MiB: usize = 1024 * 1024;
pub const REGULAR_PAGE_ALIGNMENT: Alignment =
    unsafe { Alignment::new_unchecked(REGULAR_PAGE_SIZE) };
pub const BIG_PAGE_ALIGNMENT: Alignment =
    unsafe { Alignment::new_unchecked(BIG_PAGE_SIZE) };
pub const HUGE_PAGE_ALIGNMENT: Alignment =
    unsafe { Alignment::new_unchecked(HUGE_PAGE_SIZE) };
pub const MEMORY_MAP_MAGIC_NUMBER: u32 = u32::from_le_bytes(*b"PAMS"); // 'SMAP' in little endian
pub const ENTRY_ADDRESS_MASK: u64 = 0x0000_ffff_ffff_f000;
pub const INIT_AREA_SIZE_BYTES: u64 = 256 * KiB as u64;
/// The 512-byte sectors in one disk read, because a disk packet can't
/// cross the end of its 64 KiB segment.
pub const KERNEL_SECTORS_PER_READ: u16 = 128;
/// The disk reads the first stage loads the kernel with.
pub const KERNEL_READS: u16 = 2;
/// The 512-byte sectors the first stage loads the kernel from.
pub const KERNEL_SECTORS: u16 = KERNEL_READS * KERNEL_SECTORS_PER_READ;
/// The top of the kernel's stack, at the end of the init area. It grows
/// down toward the end of the kernel image.
pub const KERNEL_STACK_TOP: u64 = INIT_AREA_SIZE_BYTES;

// The buddy allocator reserves the init area, so the kernel and its stack
// must lie in it.
const _: () = assert!(
    super::KERNEL_OFFSET + KERNEL_SECTORS as u64 * 512 < KERNEL_STACK_TOP,
    "the init area holds the whole kernel below its stack"
);
