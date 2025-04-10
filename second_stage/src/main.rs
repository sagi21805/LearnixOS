#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]
use constants::{
    enums::Sections,
    addresses::KERNEL_OFFSET
};
use core::{
    panic::PanicInfo,
    arch::asm,
};
use utils::structures::{global_descriptor_table::{GlobalDescriptorTable, GlobalDescriptorTableRegister32}, paging};

static GLOBAL_DESCRIPTOR_TABLE_LONG_MODE: GlobalDescriptorTable = GlobalDescriptorTable::long_mode(); 
static GLOBAL_DESCRIPTOR_TABLE_REGISTER_LONG: GlobalDescriptorTableRegister32 = GlobalDescriptorTableRegister32 {
    limit: 24 - 1,
    base: &GLOBAL_DESCRIPTOR_TABLE_LONG_MODE as *const GlobalDescriptorTable,
};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn _start() -> ! {
    // panic!("test");
    asm!(
        "mov eax, 0x10",
        "mov ds, eax",
    );
    paging::enable();
    asm!(
        "lgdt [{}]",
        in(reg) &GLOBAL_DESCRIPTOR_TABLE_REGISTER_LONG,
    );
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
