use core::{ascii::Char, fmt::Debug};

use common::{
    enums::{AtaCommand, FisType},
    volatile::Volatile,
};
use macros::ro_flag;

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct RegisterH2D {
    fis_type: Volatile<FisType>,
    pm_flags: Volatile<u8>,
    command: Volatile<AtaCommand>,
    features: Volatile<u8>,
    lba1: Volatile<u8>,
    lba2: Volatile<u8>,
    lba3: Volatile<u8>,
    device: Volatile<u8>,
    lba4: Volatile<u8>,
    lba5: Volatile<u8>,
    lba6: Volatile<u8>,
    features_ext: Volatile<u8>,
    sector_count: Volatile<u8>,
    sector_count_ext: Volatile<u8>,
    _reserved0: u8,
    control: Volatile<u8>,
    _reserved1: [u8; 4],
}

impl RegisterH2D {
    pub fn new(
        pm_flags: u8,
        command: AtaCommand,
        features: u16,
        lba: u64,
        device: u8,
        sector_count: u16,
        control: u8,
    ) -> RegisterH2D {
        let features_low = Volatile::new(features as u8);
        let features_ext = Volatile::new((features >> 8) as u8);
        let lba1 = Volatile::new(lba as u8);
        let lba2 = Volatile::new((lba >> 8) as u8);
        let lba3 = Volatile::new((lba >> 16) as u8);
        let lba4 = Volatile::new((lba >> 24) as u8);
        let lba5 = Volatile::new((lba >> 32) as u8);
        let lba6 = Volatile::new((lba >> 40) as u8);
        let sector_count_low = Volatile::new(sector_count as u8);
        let sector_count_ext = Volatile::new((sector_count >> 8) as u8);
        RegisterH2D {
            fis_type: Volatile::new(FisType::RegisterFisHost2Device),
            pm_flags: Volatile::new(pm_flags),
            command: Volatile::new(command),
            features: features_low,
            lba1,
            lba2,
            lba3,
            device: Volatile::new(device),
            lba4,
            lba5,
            lba6,
            features_ext,
            sector_count: sector_count_low,
            sector_count_ext,
            _reserved0: 0,
            control: Volatile::new(control),
            _reserved1: [0; 4],
        }
    }
}

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct RegisterD2H {
    fis_type: Volatile<FisType>,
    pm_flags: Volatile<u8>,
    status: Volatile<u8>,
    error: Volatile<u8>,
    lba1: Volatile<u8>,
    lba2: Volatile<u8>,
    lba3: Volatile<u8>,
    device: Volatile<u8>,
    lba4: Volatile<u8>,
    lba5: Volatile<u8>,
    lba6: Volatile<u8>,
    _reserved0: u8,
    sector_count: Volatile<u8>,
    sector_count_ext: Volatile<u8>,
    _reserved1: [u8; 6],
}

impl RegisterD2H {}

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct DmaActivateD2H {
    fis_type: Volatile<FisType>,
    pm_flags: Volatile<u8>,
    _reserved: [u8; 2],
}

/// Bidirectional
#[repr(C, align(4))]
#[derive(Clone, Copy, Debug)]
pub struct DmaSetup {
    fis_type: Volatile<FisType>,
    pm_flags: Volatile<u8>,
    _reserved0: [u8; 2],
    dma_buffer_id_lower: Volatile<u32>,
    dma_buffer_id_upper: Volatile<u32>,
    _reserved1: u32,
    dma_buffer_offset: Volatile<u32>,
    dma_transfer_count: Volatile<u32>,
    _reserved: u32,
}

/// Bidirectional
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BistActivate {
    fis_type: Volatile<FisType>,
    pm_flags: Volatile<u8>,
    pattern_def: Volatile<u8>,
    _reserved: u8,
    data1: Volatile<u8>,
    data2: Volatile<u8>,
    data3: Volatile<u8>,
    data4: Volatile<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PioSetupD2H {
    fis_type: Volatile<FisType>,
    pm_flags: Volatile<u8>,
    status: Volatile<u8>,
    error: Volatile<u8>,
    lba1: Volatile<u8>,
    lba2: Volatile<u8>,
    lba3: Volatile<u8>,
    device: Volatile<u8>,
    lba4: Volatile<u8>,
    lba5: Volatile<u8>,
    lba6: Volatile<u8>,
    _reserved0: u8,
    sector_count: Volatile<u8>,
    sector_count_exp: Volatile<u8>,
    _reserved1: u8,
    estatus: Volatile<u8>,
    transfer_count: Volatile<u16>,
    _reserved2: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Data<const SIZE: usize> {
    fis_type: Volatile<u8>,
    pm_port: Volatile<u8>,
    _reserved0: [u8; 2],
    data: Volatile<[u32; SIZE]>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SetDeviceBits {
    fis_type: Volatile<FisType>,
    pm_port: Volatile<u8>,
    status: Volatile<u8>,
    error: Volatile<u8>,
    _reserved: u32,
}

impl SetDeviceBits {
    pub fn status_low(&self) -> u8 {
        self.status.read() & !0x7
    }

    pub fn status_high(&self) -> u8 {
        (self.status.read() >> 4) & !0x7
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

pub struct GeneralInfo(u16);

impl GeneralInfo {
    ro_flag!(non_magnetic, 15);
    ro_flag!(removable_media, 7);
    ro_flag!(not_removable_media, 6);
}

impl Debug for GeneralInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Non Magnetic: {:?}", self.is_non_magnetic())?;
        writeln!(f, "Removable Media: {:?}", self.is_removable_media())?;
        writeln!(
            f,
            "Not Removable Media: {:?}",
            self.is_not_removable_media()
        )
    }
}

pub struct DeviceCapabilities(u16);

impl DeviceCapabilities {
    ro_flag!(lba_dma_support, 10);
}

impl Debug for DeviceCapabilities {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "LBA & DMA Support: {:?},", self.is_lba_dma_support())
    }
}

pub struct ValidFields(u16);

impl ValidFields {
    ro_flag!(valid_54_58, 0);
    ro_flag!(valid_64_70, 1);
}

#[derive(Debug)]
#[repr(C, align(512))]
pub struct IdentityPacketData {
    pub info: GeneralInfo,
    pub cylinders: u16,
    _reserved0: u16,
    pub heads: u16,
    _vendor0: [u16; 2],
    pub sectors: u16,
    _vendor1: [u16; 3],
    pub serial_number: [Char; 20],
    _vendor2: [u16; 3],
    /// Firmware revision in ASCII Characters
    pub firmware_rev: [Char; 8],
    /// Model number in ASCII Characters
    pub model_num: [Char; 40],
    pub max_sectors_rw_multiple: u8,
    pub _vendor3: u8,
    _reserved1: u16,
    pub capabilities: u16,
    _reserved9: u16,
    pub pio_data_transfer_time: u16,
    pub dma_data_transfer_time: u16,
    pub valid_fields: u16,
    pub cur_cylinders: u16,
    pub cur_heads: u16,
    pub cur_sectors: u16,
    pub capacity_sectors: [u16; 2],
    pub _reserved10: u16,
    pub lba_total_sectors_28: [u16; 2],
    // _reserved2: [u16; 19],
    // pub major_version: u16,
    // pub minor_version: u16,

    // pub command_sets_supported: [u16; 3],
    // pub command_sets_enabled: [u16; 3],
    // pub udma_modes: u16,
    // pub lba_total_sectors_48: u64,
    // _reserved4: [u16; 113], // Words 169-206
    // pub physical_logical_sector_size: u16, // Word 209
    // _reserved5: [u16; 7],   // Words 210-216
    // pub nominal_media_rotation_rate: u16, /* Word 217 (The SSD vs
    // HDDkey)
    //  * _reserved6: [u16; 40], */
}
