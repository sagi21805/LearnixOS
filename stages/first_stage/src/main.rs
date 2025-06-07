#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![feature(naked_functions)]
mod disk;

use common::constants::{
    addresses::{DISK_NUMBER_OFFSET, MEMORY_MAP_LENGTH, MEMORY_MAP_OFFSET, SECOND_STAGE_OFFSET},
    enums::{Interrupts, Sections, Video, VideoModes},
    values::MEMORY_MAP_MAGIC_NUMBER,
};
use core::{
    arch::{asm, global_asm, naked_asm},
    panic::PanicInfo,
};
use cpu_utils::structures::global_descriptor_table::GlobalDescriptorTable;
use disk::DiskAddressPacket;

static GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTable = GlobalDescriptorTable::protected_mode();

global_asm!(include_str!("../asm/boot.s"));

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Read the disk number the sofware was booted from
    let disk_number = unsafe { core::ptr::read(DISK_NUMBER_OFFSET as *const u8) };

    // Create a disk packet which will load 128 sectors (512 bytes each) from the disk to memory address 0x7e00
    let dap = DiskAddressPacket::new(
        128, // Max 128
        0, 0x7e0, 1,
    );
    dap.load(disk_number);

    unsafe {
        // Enter VGA text mode
        asm!(
            "mov ah, {0}",
            "mov al, {1}",
            "int {2}",
            const Video::SetMode as u8,
            const VideoModes::VGA_TX_80X25_PB_9X16_PR_720X400 as u8,
            const Interrupts::VIDEO as u8
        );

        // Obtain memory map
        obtain_memory_map();

        // Load Global Descriptor Table
        GLOBAL_DESCRIPTOR_TABLE.load();

        // Set the Protected Mode bit and enter Protected Mode
        asm!(
            "mov eax, cr0",
            "or eax, 1",
            "mov cr0, eax",
            options(readonly, nostack, preserves_flags)
        );

        // Jump to the next stage
        asm!(
            "ljmp ${section}, ${next_stage}",
            section = const Sections::KernelCode as u8,
            next_stage = const SECOND_STAGE_OFFSET, // Change this to the correct address
            options(att_syntax)
        );
    }

    loop {}
}

#[naked]
#[unsafe(no_mangle)]
pub extern "C" fn obtain_memory_map() {
    unsafe {
        naked_asm!(
            // Save all the registers.
            include_str!("../asm/memory_map.s"),
            len_address = const MEMORY_MAP_LENGTH,
            map_address = const MEMORY_MAP_OFFSET,
            smap = const MEMORY_MAP_MAGIC_NUMBER,
            function_code = const 0xE820,
            region_size = const 24
        );
    }
}
#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
