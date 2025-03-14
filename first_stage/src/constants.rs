// use crate::global_descritor_table::{GlobalDescriptorTable, GlobalDescriptorTableRegister32};

// #[unsafe(link_section = ".first_stage")]
// pub static GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTable = GlobalDescriptorTable::default();

// pub static GDTR: GlobalDescriptorTableRegister32 = GlobalDescriptorTableRegister32 {
//     limit: 0x24 - 1,
//     base: &GLOBAL_DESCRIPTOR_TABLE
// };

pub const FIRST_STAGE_OFFSET: u16 = 0x7c00;
pub const MASTER_BOOT_RECORD_OFFSET: u16 = FIRST_STAGE_OFFSET + 446;
pub const SECOND_STAGE_OFFSET: u16 = FIRST_STAGE_OFFSET + 512;
