#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]

mod bios_enums;
mod screen;
mod disk;
mod constants;

use bios_enums::PacketSize;
use core::{arch::global_asm, panic::PanicInfo};
use disk::{DiskAddressPacket, MasterBootRecord};
use constants::{MESSAGE, SECOND_STAGE_OFFSET, MASTER_BOOT_RECORD_OFFSET, BOOTABLE, NOT_BOOTABLE};
use screen::MinimalWriter;

global_asm!(include_str!("../asm/boot.s"));

#[no_mangle] 
#[link_section = ".boot"]
pub extern "C" fn first_stage() -> ! {
    
    MinimalWriter::print(MESSAGE);
    let mbr = unsafe { 
        (MASTER_BOOT_RECORD_OFFSET as *mut MasterBootRecord).as_mut_unchecked() 
    };
    let packet = DiskAddressPacket::new(
        PacketSize::Default,
        128, 
        SECOND_STAGE_OFFSET as u32, 
        1
    );
    packet.load();
    second_stage(mbr);
    

    loop {}
}

#[no_mangle]
#[optimize(none)]
#[link_section = ".second_stage"]
pub extern "C" fn second_stage(mbr: &mut MasterBootRecord) {
    MinimalWriter::print("Entering Second Stage");
    for entry in mbr.entries.iter() {
        if entry.bootable == 0x80 {
            MinimalWriter::print(BOOTABLE);
        } 
        else {
            MinimalWriter::print(NOT_BOOTABLE);
        }
    }
}


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // print(_info.message().as_str().unwrap());
    loop {}
}