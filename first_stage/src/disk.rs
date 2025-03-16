use enums::{Disk, Interrupts, PacketSize};
use core::arch::asm;

#[repr(C, packed)]
pub struct DiskAddressPacket {
    /// The size of the packte
    packet_size: u8,

    /// Zero
    zero: u8,

    /// How many sectors to read
    num_of_blocks: u16,

    /// Which address in memory to save the data
    memory_buffer: u16,

    /// Memory segment for the address
    segment: u16,

    /// The LBA address of the first sector
    abs_block_num: u64,
}

impl DiskAddressPacket {

    pub const fn new(
        num_of_blocks: u16,
        memory_buffer: u16,
        segment: u16,
        abs_block_num: u64,
    ) -> Self {
        Self {
            packet_size: PacketSize::Default as u8,
            zero: 0,
            num_of_blocks,
            memory_buffer,
            segment,
            abs_block_num,
        }
    }

    pub fn load(&self) {
        unsafe {
            asm!(
                "push si",     // si register is required for llvm it's content needs to be saved
                "mov si, {3:x}",
                "mov ah, {0}",
                "mov dl, {1}",
                "int {2}",
                "pop si",
                const Disk::ExtendedRead as u8,
                const 0x80u8,
                const Interrupts::DISK as u8,
                in(reg) self as *const Self as u32,
            )
        }
    }
}

pub fn chs_to_lba() {
    todo!()
}
