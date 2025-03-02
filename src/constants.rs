#[link_section = ".boot"]
pub static  MESSAGE: &str = "Loading partition table!\n";

#[link_section = ".boot"]
pub static  FIRST_STAGE_OFFSET: u16 = 0x7c00;

#[link_section = ".boot"]
pub static  MASTER_BOOT_RECORD_OFFSET: u16 = FIRST_STAGE_OFFSET + 446;

#[link_section = ".boot"]
pub static SECOND_STAGE_OFFSET: u16 = FIRST_STAGE_OFFSET + 512; 

#[link_section = ".second_stage"]
pub static BOOTABLE: &str = "Partion {} is bootable\n";

#[link_section = ".second_stage"]
pub static NOT_BOOTABLE: &str = "Partion {} is bootable\n";

#[link_section = ".screen"]
pub static SCREEN_WIDTH: usize  = 80;

#[link_section = ".screen"]
pub static SCREEN_HEIGHT: usize = 25;

#[link_section = ".screen"]
pub static mut SCREEN: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT] = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
