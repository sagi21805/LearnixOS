use common::enums::{BiosInterrupts, Disk};
use core::arch::asm;

// ANCHOR: dap
// The `repr(C)` means that the layout in memory will be as
// specified (like in C) because rust ABI doesn't state that
// this is promised.
//
// The `repr(packed) states that there will no padding due
// to alignment
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
// ANCHOR_END: dap

impl DiskAddressPacket {
    /// Create a new Disk Packet
    ///
    /// # Parameters
    ///
    /// - `num_of_sectors`: The number of sectors to load (Max 128)
    /// - `memory_address`: The starting memory address of the segment to
    ///   load the sectors to
    /// - `segment`: The memory segment start address
    /// - `abs_block_num`: The starting sector Logical Block Address (LBA)
    // ANCHOR: new
    pub fn new(
        num_of_sectors: u16,
        memory_address: u16,
        segment: u16,
        abs_block_num: u64,
    ) -> Self {
        Self {
            // The size of the packet
            packet_size: size_of::<Self>() as u8,
            // zero
            zero: 0,
            // Number of sectors to read, this can be a max of 128 sectors.
            // This is because the address increments every time we read a
            // sector. The largest number a register in this
            // mode can hold is 2^16 When divided by a sector
            // size, we get that we can read only 128 sectors.
            num_of_sectors: num_of_sectors.min(128),
            // The initial memory address
            memory_address,
            // The segment the memory address is in
            segment,
            // The starting LBA address to read from
            abs_block_num,
        }
    }
    // ANCHOR_END: new

    /// Load the sectors specified in the disk packet to the
    /// given memory segment
    ///
    /// # Parameters
    ///
    /// - `disk_number`: The disk number to read the sectors from
    // ANCHOR: load
    pub fn load(&self, disk_number: u8) {
        unsafe {
            // This is an inline assembly block
            // This block's assembly will be injected to the function.
            asm!(
                // si register is required for llvm it's content needs to be saved
                "push si",
                // Set the packet address in `si` and format it for a 16bit register
                "mov si, {0:x}",
                // Put function code in `ah`
                "mov ah, {1}",
                // Put disk number in `dl`
                "mov dl, {2}",
                // Call the `disk interrupt`
                "int {3}",
                // Restore si for llvm internal use.
                "pop si",
                in(reg) self as *const Self as u16,
                const Disk::ExtendedRead as u8,
                in(reg_byte) disk_number,
                const BiosInterrupts::Disk as u8,
            )
        }
    }
    // ANCHOR_END: load
}
