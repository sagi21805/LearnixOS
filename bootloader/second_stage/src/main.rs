#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]
use common::{
    constants::{KERNEL_STACK_TOP, addresses::KERNEL_OFFSET},
    enums::Sections,
};
use core::{arch::asm, panic::PanicInfo};
use x86::structures::global_descriptor_table::GlobalDescriptorTableLong;

static GLOBAL_DESCRIPTOR_TABLE_LONG_MODE: GlobalDescriptorTableLong =
    GlobalDescriptorTableLong::default();

/// The second boot stage, which enters long mode and jumps to the kernel.
///
/// # Safety
///
/// Only the first stage may jump here, in protected mode with its global
/// descriptor table loaded.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub unsafe extern "C" fn second_stage() -> ! {
    // Set data segment register
    unsafe { asm!("mov eax, 0x10", "mov ds, eax",) };
    // Enable paging and load page tables with an identity
    // mapping
    #[cfg(target_arch = "x86")]
    x86::structures::paging::enable();
    // Load the global descriptor table for long mode
    unsafe { GLOBAL_DESCRIPTOR_TABLE_LONG_MODE.load() };
    // Move to the kernel's own stack, then update global descriptor table
    // to enable long mode and jump to kernel code
    unsafe {
        asm!(
            "mov ${stack}, %esp",
            "ljmp ${section}, ${next_stage}",
            stack = const KERNEL_STACK_TOP,
            section = const Sections::KernelCode as u8,
            next_stage = const KERNEL_OFFSET,
            options(att_syntax)
        );
    }

    #[expect(
        clippy::empty_loop,
        reason = "the far jump above never returns"
    )]
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
