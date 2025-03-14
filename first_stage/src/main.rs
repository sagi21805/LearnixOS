#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]

mod bios_enums;
mod constants;
mod disk;
mod screen;
mod protected_mode;
mod global_descritor_table;
use bios_enums::PacketSize;
use constants::{SECOND_STAGE_OFFSET};
global_asm!(include_str!("../asm/boot.s"));

use bios_enums::*;
use core::{arch::{asm, global_asm}, panic::{self, PanicInfo}};
use disk::DiskAddressPacket;
use screen::MinimalWriter;


#[unsafe(no_mangle)] 
#[unsafe(link_section = ".start")]
pub extern "C" fn first_stage() -> ! {
    
    let dap = DiskAddressPacket::new(
        128, // Max 128
        0, 
        0x7e0,
        1
    );
    dap.load();
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
    protected_mode::enter_protected_mode_and_jump_to_stage_3(SECOND_STAGE_OFFSET);
    
    loop {}
}

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {
        
    }
}

