#[repr(u8)]
pub enum BiosInterrupts {
    VIDEO = 0x10,
    DISK = 0x13,
    MEMORY = 0x15,
}

#[repr(u8)]
pub enum Video {
    SetMode = 0x0,
}

#[repr(u8)]
pub enum Disk {
    ExtendedRead = 0x42,
}

#[repr(u16)]
pub enum Memory {
    MemoryMap = 0xe820,
}

#[repr(u16)]
pub enum MemoryRegionSize {
    Regular = 20,
    Extended = 24,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MemoryRegionType {
    Unknown = 0u32,
    Usable = 0x1u32,
    Reserved = 0x2u32,
    Reclaimable = 0x3u32,
    ACPINVS = 0x4u32,
    BadMemory = 0x5u32,
}

#[repr(u8)]
pub enum DiskPacketSize {
    Default = 0x10,
}

#[repr(u8)]
#[allow(non_camel_case_types)]
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
