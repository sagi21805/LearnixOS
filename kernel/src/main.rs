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
use core::{
    alloc::{GlobalAlloc, Layout},
    num::NonZero,
    panic::PanicInfo,
};

use crate::{
    drivers::{
        interrupt_handlers,
        keyboard::{KEYBOARD, keyboard::Keyboard},
        pci::PciConfigurationCycle,
        pic8259::{CascadedPIC, PIC},
        vga_display::color_code::Color,
    },
    memory::memory_map::{ParsedMapDisplay, parse_map},
};

use common::{
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    error::PciConfigurationError,
};
use cpu_utils::{
    instructions::interrupts::{self, hlt},
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
        let idt_address = ALLOCATOR
            .assume_init_ref()
            .alloc(Layout::from_size_align_unchecked(
                REGULAR_PAGE_SIZE,
                REGULAR_PAGE_ALIGNMENT.as_usize(),
            )) as usize;
        InterruptDescriptorTable::init(&mut IDT, idt_address.into());
        ok_msg!("Initialized interrupt descriptor table");
        interrupt_handlers::init(IDT.assume_init_mut());
        ok_msg!("Initialized interrupts handlers");
        CascadedPIC::init(&mut PIC);
        ok_msg!("Initialized Programmable Interrupt Controller");
        let keyboard_buffer_address =
            ALLOCATOR
                .assume_init_ref()
                .alloc(Layout::from_size_align_unchecked(
                    REGULAR_PAGE_SIZE,
                    REGULAR_PAGE_ALIGNMENT.as_usize(),
                )) as usize;
        Keyboard::init(
            &mut KEYBOARD,
            keyboard_buffer_address.into(),
            NonZero::new(REGULAR_PAGE_SIZE).unwrap(),
        );
        ok_msg!("Initialized Keyboard");
        interrupts::enable();
    }
    for bus in 0..=255 {
        for device in 0..32 {
            let header = match PciConfigurationCycle::read_common_header(bus, device) {
                Ok(h) => h,
                Err(PciConfigurationError::NonExistentDevice(_, _)) => {
                    continue;
                }
                Err(e) => {
                    println!("Error! {:?}", e);
                    continue;
                }
            };
            println!("{:?}", header)
        }
    }

    loop {
        unsafe {
            let char = KEYBOARD.assume_init_mut().read_char();
            if char != "" {
                print!("{}", char);
            } else {
                hlt();
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
