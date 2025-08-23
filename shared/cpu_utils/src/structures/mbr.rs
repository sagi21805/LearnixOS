#[repr(C, packed)]
pub struct PartitionTableEntry {
    /// Boot indicator bit flag: 0 = no, 0x80 = bootable (or "active").
    pub bootable: u8,

    /// Starting head of the partition.
    pub start_head: u8,

    /// Bits 0-5 are the starting sector.   
    /// Bits 6-16 are the starting cylinder.
    pub sector_cylinder_start: u16,

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
