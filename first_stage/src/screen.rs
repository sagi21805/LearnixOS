use crate::bios_enums::*;
use core::arch::asm;

pub struct MinimalWriter;

impl MinimalWriter {
    #[unsafe(link_section = ".first_stage")]
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

    #[unsafe(link_section = ".first_stage")]
    pub fn print(str: &str) {
        for char in str.bytes() {
            MinimalWriter::print_char(char);
        }
    }

    #[unsafe(link_section = ".first_stage")]
    pub fn set_vga_text_mode() {
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
