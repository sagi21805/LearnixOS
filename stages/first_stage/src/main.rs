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

/// Bootloader entry point that loads the next stage and transitions to protected mode.
///
/// This function is the initial entry point for the bootloader. It loads disk sectors containing the next boot stage into memory, sets the VGA text mode, retrieves the system memory map, loads the Global Descriptor Table (GDT), enables protected mode, and performs a far jump to the next stage. Execution does not return from this function; if the jump fails, it enters an infinite loop.
///
/// # Safety
///
/// This function performs low-level hardware operations, uses inline assembly, and transitions the CPU into protected mode. It must only be called as the bootloader entry point by the system firmware.
///
/// # Panics
///
/// This function does not return and will enter an infinite loop if execution continues past the protected mode jump.
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

/// Retrieves the system memory map using BIOS interrupt 0xE820.
///
/// This naked function executes assembly code to query the BIOS for the system's memory regions,
/// storing the results at a predefined memory location. The memory map is used to identify usable
/// and reserved memory areas for subsequent boot stages. The function preserves all registers and
/// does not return any value.
///
/// # Safety
///
/// This function must only be called in real mode before entering protected mode, as it relies on BIOS interrupts.
///
/// # Examples
///
/// ```no_run
/// // Called during bootloader initialization
/// obtain_memory_map();
/// ```
#[unsafe(naked)]
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
            extended_region_size = const 24,
            regular_region_size = const 20
        );
    }
}
/// Halts execution indefinitely on panic.
///
/// This panic handler enters an infinite loop, effectively stopping the system when a panic occurs in a no_std environment.
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
