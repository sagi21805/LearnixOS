// ANCHOR: bios_interrupts
#[repr(u8)]
/// BIOS interrupts number for each interrupt type used in
/// the kernel.
pub enum BiosInterrupts {
    Video = 0x10,
    Disk = 0x13,
    Memory = 0x15,
}
// ANCHOR_END: bios_interrupts

// ANCHOR: video_interrupts
#[repr(u8)]
/// Video interrupt number for each function used in the
/// kernel.
pub enum VideoInterrupt {
    SetMode = 0x0,
}
// ANCHOR_END: video_interrupts

// ANCHOR: disk_interrupts
#[repr(u8)]
/// Disk interrupt number for each function used in the
/// kernel.
pub enum DiskInterrupt {
    ExtendedRead = 0x42,
}
// ANCHOR_END: disk_interrupts

// ANCHOR: memory_interrupts
#[repr(u16)]
/// Memory interrupt number for each function used in the
/// kernel.
pub enum MemoryInterrupt {
    MemoryMap = 0xe820,
}
// ANCHOR_END: memory_interrupts

// ANCHOR: memory_region_size
#[repr(u16)]
/// Memory region size for the memory map.
pub enum MemoryRegionSize {
    Regular = 20,
    Extended = 24,
}
// ANCHOR_END: memory_region_size

// ANCHOR: memory_region_type
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Memory region type for the memory map.
pub enum MemoryRegionType {
    Usable = 1,
    Reserved = 2,
    Reclaimable = 3,
    ACPINVS = 4,
    BadMemory = 5,
    // Custom attributes for added entries
    UserEnterd = 6,
    Filler = 7,
}
// ANCHOR_END: memory_region_type

// ANCHOR: video_modes
#[repr(u8)]
#[allow(non_camel_case_types)]
/// Video modes supported by the kernel.
pub enum VideoModes {
    /// VGA Common Text Mode ->
    ///
    /// Text resolution 80x25
    ///
    /// PixelBox resolution 9x16
    ///
    /// Pixel resolution 720x400
    VGA_TX_80X25_PB_9X16_PR_720X400 = 0x3,
}
// ANCHOR_END: video_modes
