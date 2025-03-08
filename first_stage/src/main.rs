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
mod macros;
mod global_descritor_table;
use bios_enums::PacketSize;
global_asm!(include_str!("../asm/boot.s"));

use constants::{
    MASTER_BOOT_RECORD_OFFSET,
    SECOND_STAGE_OFFSET,
    KERNEL_START,
    BOOTABLE,
    NOT_BOOTABLE,
};
use core::{arch::global_asm, panic::PanicInfo};
use disk::{DiskAddressPacket, MasterBootRecord};
use screen::MinimalWriter;

first_stage! {
    #[no_mangle] 
    pub extern "C" fn first_stage() -> ! {
        
        let mbr = unsafe { 
            (MASTER_BOOT_RECORD_OFFSET as *mut MasterBootRecord).as_mut_unchecked() 
        };
        let packet = DiskAddressPacket::new(
            PacketSize::Default,
            8, // 0x1000 first bytes
            SECOND_STAGE_OFFSET as u32, 
            1
        );
        packet.load();
        MinimalWriter::print("Hello World");
        // protected_mode::enter_protected_mode(&mbr);
        // second_stage_entry(mbr);
        
        loop {}
    }
}

