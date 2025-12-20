use common::enums::{AtaCommand, FisType};

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct RegisterH2D {
    fis_type: FisType,
    pm_flags: u8,
    command: AtaCommand,
    features: u8,
    lba1: u8,
    lba2: u8,
    lba3: u8,
    device: u8,
    lba4: u8,
    lba5: u8,
    lba6: u8,
    features_ext: u8,
    sector_count: u8,
    sector_count_ext: u8,
    _resvered0: u8,
    control: u8,
    _reserved1: [u8; 4],
}

impl RegisterH2D {}

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct RegisterD2H {
    fis_type: FisType,
    pm_flags: u8,
    status: u8,
    error: u8,
    lba1: u8,
    lba2: u8,
    lba3: u8,
    device: u8,
    lba4: u8,
    lba5: u8,
    lba6: u8,
    _reserved0: u8,
    sector_count: u8,
    sector_count_ext: u8,
    _reserved1: [u8; 6],
}

impl RegisterD2H {}

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct DmaActivateD2H {
    fis_type: FisType,
    pm_flags: u8,
    _reserved: [u8; 2],
}

/// Bidirectional
#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct DmaSetup {
    fis_type: FisType,
    pm_flags: u8,
    _reserved0: [u8; 2],
    dma_buffer_id_lower: u32,
    dma_buffer_id_upper: u32,
    _reserved1: u32,
    dma_buffer_offset: u32,
    dma_transfer_count: u32,
    _reserved: u32,
}

/// Bidirectional
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BistActivate {
    fis_type: FisType,
    pm_flags: u8,
    pattern_def: u8,
    _reserved: u8,
    data1: u8,
    data2: u8,
    data3: u8,
    data4: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PioSetupD2H {
    fis_type: FisType,
    pm_flags: u8,
    status: u8,
    error: u8,
    lba1: u8,
    lba2: u8,
    lba3: u8,
    device: u8,
    lba4: u8,
    lba5: u8,
    lba6: u8,
    _reserved0: u8,
    sector_count: u8,
    sector_count_exp: u8,
    _reserved1: u8,
    estatus: u8,
    transfer_count: u16,
    _reserved2: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Data<const SIZE: usize> {
    fis_type: u8,
    pm_port: u8,
    _reserved0: [u8; 2],
    data: [u32; SIZE],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SetDeviceBits {
    fis_type: FisType,
    pm_port: u8,
    status: u8,
    error: u8,
    _reserved: u32,
}

impl SetDeviceBits {
    pub fn status_low(&self) -> u8 {
        self.status & !0x7
    }

    pub fn status_high(&self) -> u8 {
        (self.status >> 4) & !0x7
    }
}

#[repr(C)]
pub union Fis {
    pub h2d: RegisterH2D,
    pub d2h: RegisterD2H,
    pub dma_activate: DmaActivateD2H,
    pub dma_setup: DmaSetup,
    pub bist: BistActivate,
    pub pio_setup: PioSetupD2H,
    pub set_device_bits: SetDeviceBits,
    pub raw: [u8; 64],
}

impl Default for Fis {
    fn default() -> Self {
        Fis { raw: [0; 64] }
    }
}

pub struct IdentityPacketData {
    data: [u16; 0x100],
}
