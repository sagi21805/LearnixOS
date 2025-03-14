use crate::screen::ScreenChar;
pub static SCREEN_WIDTH: usize = 80;
pub static SCREEN_HEIGHT: usize = 25;
pub static BOOTABLE: &str = "Partition is bootable";
pub static NOT_BOOTABLE: &str = "Partition is NOT bootable";
pub const VGA_BUFFER_PTR: *mut ScreenChar = 0xb8000 as *mut ScreenChar;
