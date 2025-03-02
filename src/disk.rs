use crate::bios_enums::PacketSize;
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
    transfer_buffer: u32,

    /// The LBA address of the first sector
    abs_block_num: u64
}

#[repr(C, packed)]
pub struct PartitionTableEntry {
    /// Boot indicator bit flag: 0 = no, 0x80 = bootable (or "active").
    pub bootable: u8,

    /// Starting head of the partition.
    pub start_head: u8,

    /// Bits 0-5 are the starting sector.   
    /// Bits 6-16 are the starting cylinder.
    pub sector_cylider_start: u16, 

    /// SystemID.  
    pub system_id: u8,

    /// Ending head of the partition.
    pub end_head: u8,

    /// Bits 0-5 are the ending sector.   
    /// Bits 6-16 are the ending cylinder.    
    pub sector_cylinder_head: u16,

    /// Relative Sector (to start of partition -- also equals the partition's starting LBA value) 
    pub relative_sector: u32,

    /// Total Sectors in partition 
    pub total_sectors: u32, 
}

pub struct MasterBootRecord {
    pub entries: [PartitionTableEntry; 4],
}

impl DiskAddressPacket {

    #[no_mangle]
    #[link_section = ".disk_minimal"]
    pub fn new(
        packet_size: PacketSize,
        num_of_blocks: u16,
        transfer_buffer: u32,
        abs_block_num: u64
    ) -> Self {

        Self {
            packet_size: packet_size as u8,
            zero: 0,
            num_of_blocks,
            transfer_buffer,
            abs_block_num
        }
    }

    #[no_mangle]
    #[link_section = ".disk_minimal"]
    pub fn load(&self) {
        let self_address = self as *const Self as u32;
        unsafe {
            asm!(
                "push si",     // si register is required for llvm it's content needs to be saved
                "mov si, {3:x}",
                "mov ah, {0}",
                "mov dl, {1}",
                "int {2}",
                "pop si",
                const 0x42u8,
                const 0x80u8,
                const 0x13u8,
                in(reg) self_address,
            )
        }

    }

    #[no_mangle]
    #[link_section = ".disk"]
    pub fn chs_to_lba() {
        todo!()
    }


}