#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]
use common::{constants::addresses::KERNEL_OFFSET, enums::Sections};
use core::{arch::asm, panic::PanicInfo};
use cpu_utils::structures::global_descriptor_table::GlobalDescriptorTableLong;

static GLOBAL_DESCRIPTOR_TABLE_LONG_MODE: GlobalDescriptorTableLong =
    GlobalDescriptorTableLong::default();

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn _start() -> ! {
    // Set data segment register
    asm!("mov eax, 0x10", "mov ds, eax",);

    // Enable paging and load page tables with an identity mapping
    #[cfg(target_arch = "x86")]
    cpu_utils::structures::paging::enable();
    // Load the global descriptor table for long mode
    GLOBAL_DESCRIPTOR_TABLE_LONG_MODE.load();
    // Update global descriptor table to enable long mode and jump to kernel code
    asm!(
        "ljmp ${section}, ${next_stage}",
        section = const Sections::KernelCode as u8,
        next_stage = const KERNEL_OFFSET,
        options(att_syntax)
    );
    loop {}
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
