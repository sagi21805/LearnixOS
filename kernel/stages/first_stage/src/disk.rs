use common::enums::{BiosInterrupts, Disk};
use core::arch::asm;

#[repr(C, packed)]
pub struct DiskAddressPacket {
    /// The size of the packet
    packet_size: u8,

    /// Zero
    zero: u8,

    /// How many sectors to read
    num_of_sectors: u16,

    /// Which address in memory to save the data
    memory_address: u16,

    /// Memory segment for the address
    segment: u16,

    /// The LBA address of the first sector
    abs_block_num: u64,
}

impl DiskAddressPacket {
    /// Create a new Disk Packet
    ///
    /// # Parameters
    ///
    /// - `num_of_sectors`: The number of sectors to load (Max 128)
    /// - `memory_address`: The starting memory address of the segment to load the sectors to
    /// - `segment`: The memory segment start address
    /// - `abs_block_num`: The starting sector Logical Block Address (LBA)
    pub fn new(num_of_sectors: u16, memory_address: u16, segment: u16, abs_block_num: u64) -> Self {
        Self {
            packet_size: size_of::<Self>() as u8,
            zero: 0,
            num_of_sectors: num_of_sectors.min(128),
            memory_address,
            segment,
            abs_block_num,
        }
    }

    /// Load the sectors specified in the disk packet to the given memory segment
    ///
    /// # Parameters
    ///
    /// - `disk_number`: The disk number to read the sectors from
    pub fn load(&self, disk_number: u8) {
        unsafe {
            asm!(
                // si register is required for llvm it's content needs to be saved
                "push si",
                "mov si, {0:x}",
                "mov ah, {1}",
                "mov dl, {2}",
                "int {3}",
                "pop si",
                in(reg) self as *const Self as u16,
                const Disk::ExtendedRead as u8,
                in(reg_byte) disk_number,
                const BiosInterrupts::DISK as u8,
            )
        }
    }
}
