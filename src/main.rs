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
// mod global_descritor_table;

use bios_enums::{Color, PacketSize};
use constants::{
    MASTER_BOOT_RECORD_OFFSET,
    MESSAGE,
    SECOND_STAGE_OFFSET,
    BOOTABLE,
    NOT_BOOTABLE,
    KERNEL_START,
};
use core::{arch::{asm, global_asm}, panic::PanicInfo};
use disk::{DiskAddressPacket, MasterBootRecord};
// use screen::{ColorCode, MinimalWriter, Writer};
use screen::{ColorCode, MinimalWriter, Writer};

global_asm!(include_str!("../asm/boot.s"));

#[no_mangle] 
#[link_section = ".boot"]
#[cfg(feature = "stage-1-2")]
pub extern "C" fn first_stage() -> ! {
    
    MinimalWriter::print(MESSAGE);
    let mbr = unsafe { 
        (MASTER_BOOT_RECORD_OFFSET as *mut MasterBootRecord).as_mut_unchecked() 
    };
    let packet = DiskAddressPacket::new(
        PacketSize::Default,
        8, 
        SECOND_STAGE_OFFSET as u32, 
        1
    );
    packet.load();
    second_stage(mbr);
    

    loop {}
}
#[no_mangle]
#[link_section = ".second_stage"]
#[cfg(feature = "stage-1-2")]
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
    let packet = DiskAddressPacket::new(
        PacketSize::Default,
        128, 
        KERNEL_START as u32, 
        9
    );
    packet.load();
    // protected_mode::enter_unreal_mode();
    protected_mode::enter_protected_mode(KERNEL_START);
}

#[no_mangle]
#[link_section = ".start"]
#[cfg(feature = "stage-3")]
pub extern "C" fn start() {
    MinimalWriter::print("Third stage");
    let mut writer = Writer::new();
    unsafe {
        asm!("nop");
    }
    // writer.print(
    //     "Hello World",
    //     ColorCode::new(Color::Blue, Color::Black)
    // );
}


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // print(_info.message().as_str().unwrap());
    loop {}
}
