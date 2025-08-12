#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
mod disk;

use common::{
    constants::{
        DISK_NUMBER_OFFSET, MEMORY_MAP_LENGTH, MEMORY_MAP_MAGIC_NUMBER, MEMORY_MAP_OFFSET,
        SECOND_STAGE_OFFSET,
    },
    enums::{BiosInterrupts, Sections, Video, VideoModes},
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
pub fn first_stage() -> ! {
    // Read the disk number the os was booted from
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
            const BiosInterrupts::VIDEO as u8
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
            "jmp ${section}, ${next_stage}",
            section = const Sections::KernelCode as u8,
            next_stage = const SECOND_STAGE_OFFSET, // Change this to the correct address
        );
    }

    loop {}
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn obtain_memory_map() {
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
#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
