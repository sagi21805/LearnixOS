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
use core::panic::PanicInfo;

use crate::{
    drivers::{
        interrupt_handlers,
        keyboard::{self, KEYBOARD_BUFFER},
        pic8259::{CascadedPIC, PIC},
    },
    memory::memory_map::{ParsedMapDisplay, parse_map},
};

use common::{
    address_types::VirtualAddress,
    constants::{IDT_OFFSET, KEYBOARD_BUFFER_OFFSET},
    enums::PS2ScanCode,
    fail_msg, ok_msg, print, println,
    vga_display::color_code::Color,
};
use cpu_utils::{
    instructions::{
        cpuid::{self, CpuFeatures},
        interrupts::{self, hlt},
    },
    structures::interrupt_descriptor_table::{IDT, InterruptDescriptorTable},
};
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
        InterruptDescriptorTable::init(&mut IDT, IDT_OFFSET.into());
        ok_msg!("Initialized interrupt descriptor table");
        interrupt_handlers::init(IDT.assume_init_mut());
        ok_msg!("Initialized interrupts handlers");
        CascadedPIC::init(&mut PIC);
        ok_msg!("Initialized Programmable Interrupt Controller");
        let val = cpuid::get_vendor_string();
        let cpu_string = core::str::from_utf8(&val);
        let features = CpuFeatures::new();
        println!("{:?}", cpu_string);
        println!("Has APIC: {}", features.has_apic());
        keyboard::init(
            &mut KEYBOARD_BUFFER,
            VirtualAddress::new_unchecked(KEYBOARD_BUFFER_OFFSET),
            0x1000,
        );
        ok_msg!("Initialized Keyboard");
        interrupts::enable();
    }

    loop {
        unsafe {
            let key = KEYBOARD_BUFFER.assume_init_mut().read();
            match key {
                Some(scan_code) => {
                    let code = PS2ScanCode::from_scancode(scan_code);
                    match code {
                        Some(c) => print!("{}", c),
                        None => print!(" code: {:x}", scan_code),
                    }
                }
                None => {
                    hlt();
                }
            }
        }
    }
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        interrupts::disable();
    }
    fail_msg!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}
