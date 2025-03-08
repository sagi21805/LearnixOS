use crate::{first_stage, screen::ScreenChar, second_stage, global_descritor_table::GlobalDescriptorTable};

first_stage! {
    pub static GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTable = GlobalDescriptorTable::default();
}
pub const FIRST_STAGE_OFFSET: u16 = 0x7c00;
pub const MASTER_BOOT_RECORD_OFFSET: u16 = FIRST_STAGE_OFFSET + 446;
pub const SECOND_STAGE_OFFSET: u16 = FIRST_STAGE_OFFSET + 512;

second_stage! {
    pub static SCREEN_WIDTH: usize = 80;
    pub static SCREEN_HEIGHT: usize = 25;
    pub static BOOTABLE: &str = "Partition is bootable";
    pub static NOT_BOOTABLE: &str = "Partition is NOT bootable";
}

pub const VGA_BUFFER_PTR: *mut ScreenChar = 0xb8000 as *mut ScreenChar;
pub const KERNEL_START: *mut u8 = 0x10000 as *mut u8;
