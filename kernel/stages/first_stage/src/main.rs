#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![deny(clippy::all)]
mod disk;

use common::{
    constants::{
        DISK_NUMBER_OFFSET, MEMORY_MAP_LENGTH, MEMORY_MAP_MAGIC_NUMBER,
        MEMORY_MAP_OFFSET, SECOND_STAGE_OFFSET,
    },
    enums::{
        BiosInterrupts, MemoryInterrupt, MemoryRegionSize, Sections,
        VideoInterrupt, VideoModes,
    },
};
use core::{
    arch::{asm, global_asm, naked_asm},
    panic::PanicInfo,
};
use cpu_utils::structures::global_descriptor_table::GlobalDescriptorTableProtected;
use disk::DiskAddressPacket;

// ANCHOR: gdt_static
static GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTableProtected =
    GlobalDescriptorTableProtected::default();
// ANCHOR_END: gdt_static

global_asm!(include_str!("../asm/boot.s"));

// ANCHOR: first_stage
#[unsafe(no_mangle)]
pub fn first_stage() -> ! {
    // Read the disk number the os was booted from
    let disk_number =
        unsafe { core::ptr::read(DISK_NUMBER_OFFSET as *const u8) };

    // Create a disk packet which will load 4 sectors (512 bytes each)
    // from the disk to memory address 0x7e00
    // The address 0x7e00 was chosen because it is exactly one sector
    //  after the initial address 0x7c00.
    let dap = DiskAddressPacket::new(
        4,     // Number of sectors
        0,     // Memory address
        0x7e0, // Memory segment
        1,     // Starting LBA address
    );
    dap.load(disk_number);
    // ANCHOR_END: first_stage
    let kernel_dap = DiskAddressPacket::new(128, 0, 0x1000, 66);
    kernel_dap.load(disk_number);
    unsafe {
        // Enter VGA text mode
        asm!(
            "mov ah, {0}",
            "mov al, {1}",
            "int {2}",
            const VideoInterrupt::SetMode as u8,
            const VideoModes::VGA_TX_80X25_PB_9X16_PR_720X400 as u8,
            const BiosInterrupts::Video as u8
        );

        // Obtain memory map
        obtain_memory_map();

        // ANCHOR: enter_protected_mode
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
        // The 'ljmp' instruction is required to because it updates the cpu
        // segment to the new ones from our GDT.
        //
        // The segment is the offset in the GDT.
        // (KernelCode = 0x10 which is the code segment)
        asm!(
            "ljmp ${segment}, ${next_stage_address}",
            segment = const Sections::KernelCode as u8,
            next_stage_address = const SECOND_STAGE_OFFSET,
            options(att_syntax)
        );
        // ANCHOR_END: enter_protected_mode
    }

    #[allow(clippy::empty_loop)]
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
        function_code = const MemoryInterrupt::MemoryMap as u16,
        extended_region_size = const MemoryRegionSize::Extended as u8,
        regular_region_size = const MemoryRegionSize::Regular as u8,
    );
}
#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
