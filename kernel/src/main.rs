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
#![feature(allocator_api)]
#![feature(never_type)]
#![feature(vec_push_within_capacity)]
mod drivers;
mod memory;
use core::{
    alloc::{AllocError, Allocator, Layout},
    num::NonZero,
    panic::PanicInfo,
};

use crate::{
    drivers::{
        interrupt_handlers,
        keyboard::{KEYBOARD, keyboard::Keyboard},
        pci::{self},
        pic8259::{CascadedPIC, PIC},
        vga_display::color_code::Color,
    },
    memory::memory_map::{ParsedMapDisplay, parse_map},
};

use common::{
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    enums::HeaderType,
};
use cpu_utils::{
    instructions::interrupts::{self},
    structures::interrupt_descriptor_table::{IDT, InterruptDescriptorTable},
};
use memory::allocators::page_allocator::{ALLOCATOR, allocator::PhysicalPageAllocator};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub unsafe extern "C" fn _start() -> Result<!, AllocError> {
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
            .allocate(Layout::from_size_align_unchecked(
                REGULAR_PAGE_SIZE,
                REGULAR_PAGE_ALIGNMENT.as_usize(),
            ))?
            .addr()
            .get()
            .into();
        InterruptDescriptorTable::init(&mut IDT, idt_address);
        ok_msg!("Initialized interrupt descriptor table");
        interrupt_handlers::init(IDT.assume_init_mut());
        ok_msg!("Initialized interrupts handlers");
        CascadedPIC::init(&mut PIC);
        ok_msg!("Initialized Programmable Interrupt Controller");
        let keyboard_buffer_address = ALLOCATOR
            .assume_init_ref()
            .allocate(Layout::from_size_align_unchecked(
                REGULAR_PAGE_SIZE,
                REGULAR_PAGE_ALIGNMENT.as_usize(),
            ))?
            .addr()
            .get()
            .into();
        Keyboard::init(
            &mut KEYBOARD,
            keyboard_buffer_address,
            NonZero::new(REGULAR_PAGE_SIZE).unwrap(),
        );
        ok_msg!("Initialized Keyboard");
        interrupts::enable();
    }
    let pci_devices = pci::scan_pci().expect("Pci devices are not initialized: ");
    println!("Press ENTER to enumerate PCI devices!");
    let a = pci_devices.as_ptr() as usize;
    println!("pci_devices address: {:x}", a);
    for device in pci_devices.iter() {
        loop {
            unsafe {
                let c = KEYBOARD.assume_init_mut().read_char();
                if c == "\n" {
                    break;
                }
            }
        }
        match device.identify() {
            HeaderType::GeneralDevice => println!("{:?}", unsafe { device.general_device }),
            _ => println!("{:#?}", unsafe { device.common }),
        }
    }
    loop {
        unsafe {
            print!("{}", KEYBOARD.assume_init_mut().read_char());
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
