use core::arch::asm;
use crate::{bios_enums::{Interrupts, Video, VideoModes}, constants::{SCREEN_HEIGHT, SCREEN_WIDTH}};

pub struct MinimalWriter;

impl MinimalWriter {

    #[no_mangle]
    #[link_section = ".screen_minimal"]
    fn print_char(char: u8) {
        unsafe {
            asm!(
                "mov ah, {0}",
                "int {1}",         
                const Video::DisplayChar as u8,
                const Interrupts::VIDEO as u8, 
                in("al") char,
                options(nostack, nomem)
            );
        }
    }
    
    #[no_mangle]
    #[link_section = ".screen_minimal"]
    pub fn print(str: &str) {
        for char in str.bytes() {
            MinimalWriter::print_char(char);
        }
    }

    #[no_mangle]
    #[link_section = ".screen"]
    pub fn set_video_mode() {
        unsafe {
            asm!(
                "mov ah, {0}",
                "mov al, {1}",
                "int {2}",
                const Video::SetMode as u8,
                const VideoModes::VGA_TX_80X25_PB_9X16_PR_720X400 as u8,
                const Interrupts::VIDEO as u8
            )
        }
    }

}

pub struct Writer {
    screen_buffer: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],

}

impl Writer {

    

}
