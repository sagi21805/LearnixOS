#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]

mod disk;

use constants::addresses::{DISK_NUMBER_OFFSET, SECOND_STAGE_OFFSET};
use constants::enums::{Interrupts, Sections, Video, VideoModes};
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use disk::DiskAddressPacket;
use utils::structures::global_descriptor_table::{GlobalDescriptorTable, GlobalDescriptorTableRegister32};

pub static GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTable = GlobalDescriptorTable::default();

pub static GDTR: GlobalDescriptorTableRegister32 = GlobalDescriptorTableRegister32 {
    limit: 24 - 1,
    base: &GLOBAL_DESCRIPTOR_TABLE as *const GlobalDescriptorTable,
};

global_asm!(include_str!("../asm/boot.s"));

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let disk_number = unsafe { core::ptr::read(DISK_NUMBER_OFFSET as *const u8) };
    let dap = DiskAddressPacket::new(
        128, // Max 128
        0, 0x7e0, 1,
    );
    dap.load(disk_number);
    unsafe {

        // Enable VGA text mode
        asm!(
            "mov ah, {0}",
            "mov al, {1}",
            "int {2}",
            const Video::SetMode as u8,
            const VideoModes::VGA_TX_80X25_PB_9X16_PR_720X400 as u8,
            const Interrupts::VIDEO as u8
        );

        // Load Global Descriptor Table
        asm!(
            "cli",
            "lgdt [{}]",
            "mov eax, cr0",
            "or eax, 1",
            "mov cr0, eax",
            in(reg) &GDTR,
            options(readonly, nostack, preserves_flags)
        );

        // Jump to code segment
        asm!(
            "ljmp ${section}, ${next_stage}",
            section = const Sections::KernelCode as u8,
            next_stage = const SECOND_STAGE_OFFSET, // Change this to the correct address
            options(att_syntax)
        );
    }

    loop {}
}

#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
