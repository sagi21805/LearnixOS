#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]
#![feature(const_trait_impl)]
#![feature(stmt_expr_attributes)]
#![feature(abi_x86_interrupt)]
#![feature(macro_metavar_expr_concat)]
mod drivers;
mod memory;
use core::{arch::asm, panic::PanicInfo};

use crate::{
    drivers::interrupt_handlers::initialize_interrupts,
    memory::memory_map::{ParsedMapDisplay, parse_map},
};

use common::constants::IDT_OFFSET;
use cpu_utils::structures::interrupt_descriptor_table::{IDT, InterruptDescriptorTable};
use drivers::vga_display::color_code::Color;
use memory::allocators::page_allocator::{ALLOCATOR, allocator::PhysicalPageAllocator};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub unsafe extern "C" fn _start() -> ! {
    ok_msg!("Entered Protected Mode");
    ok_msg!("Enabled Paging");
    ok_msg!("Entered Long Mode");
    parse_map();
    ok_msg!("Obtained Memory Map");
    println!("{}", ParsedMapDisplay(parsed_memory_map!()));
    PhysicalPageAllocator::init(unsafe { &mut ALLOCATOR });
    ok_msg!("Allocator Initialized");
    unsafe {
        InterruptDescriptorTable::init(&mut IDT, IDT_OFFSET as *mut InterruptDescriptorTable);
        initialize_interrupts(IDT.assume_init_mut());

        // For now, disable pic interrupts and enable interrupts
        asm!("mov al, 0xff", "out 0x21, al", "out 0xA1, al");
        asm!("sti");
    }
    ok_msg!("Initialized interrupt descriptor table");

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    fail_msg!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}
