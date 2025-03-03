use crate::screen::ScreenChar;

#[link_section = ".boot"]
#[cfg(feature = "stage-1-2")]
pub static MESSAGE: &str = "Loading partition table\n";

#[link_section = ".boot"]
#[cfg(feature = "stage-1-2")]
pub static FIRST_STAGE_OFFSET: u16 = 0x7c00;

#[link_section = ".boot"]
#[cfg(feature = "stage-1-2")]
pub static MASTER_BOOT_RECORD_OFFSET: u16 = FIRST_STAGE_OFFSET + 446;

#[link_section = ".boot"]
#[cfg(feature = "stage-1-2")]
pub static SECOND_STAGE_OFFSET: u16 = FIRST_STAGE_OFFSET + 512;

#[link_section = ".second_stage"]
#[cfg(feature = "stage-1-2")]
pub static BOOTABLE: &str = "Partion {} is bootable\n";

#[link_section = ".second_stage"]
#[cfg(feature = "stage-1-2")]
pub static NOT_BOOTABLE: &str = "Partion {} is NOT fbootable\n";

#[link_section = ".kernel_function"]
pub static SCREEN_WIDTH: usize = 80;

#[link_section = ".kernel_function"]
pub static SCREEN_HEIGHT: usize = 25;

pub const VGA_BUFFER_PTR: *mut ScreenChar = 0xb8000 as *mut ScreenChar;

pub const KERNEL_START: *mut u8 = 0x0010_0000 as *mut u8;
