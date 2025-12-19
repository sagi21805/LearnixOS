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
#![feature(const_default)]
#![feature(ascii_char_variants)]
#![feature(ascii_char)]
#![feature(const_convert)]
#![deny(clippy::all)]
mod drivers;
mod memory;
use core::{
    alloc::{Allocator, Layout},
    mem::MaybeUninit,
    num::NonZero,
    panic::PanicInfo,
};

use crate::{
    drivers::{
        ata::ahci::{
            AhciDeviceController, GenericHostControl, HBAMemoryRegisters,
        },
        interrupt_handlers,
        keyboard::{KEYBOARD, ps2_keyboard::Keyboard},
        pci::{self},
        pic8259::{CascadedPIC, PIC},
        vga_display::color_code::ColorCode,
    },
    memory::{
        allocators::page_allocator::{
            allocator::PhysicalPageAllocator,
            extensions::PhysicalAddressExt,
        },
        memory_map::{ParsedMapDisplay, parse_map},
    },
};

use common::{
    address_types::PhysicalAddress,
    constants::{
        BIG_PAGE_ALIGNMENT, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    enums::{
        Color, DeviceDetection, DeviceType, InterfacePowerManagement,
        PS2ScanCode, PageSize, PciDeviceType,
    },
};
use cpu_utils::{
    instructions::interrupts::{self},
    structures::{
        interrupt_descriptor_table::{IDT, InterruptDescriptorTable},
        paging::PageEntryFlags,
    },
};

use memory::allocators::page_allocator::ALLOCATOR;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _start() -> ! {
    okprintln!("Entered Protected Mode");
    okprintln!("Enabled Paging");
    okprintln!("Entered Long Mode");
    parse_map();
    okprintln!("Obtained Memory Map");
    println!("{}", ParsedMapDisplay(parsed_memory_map!()));
    PhysicalPageAllocator::init(unsafe { &mut ALLOCATOR });
    okprintln!("Allocator Initialized");
    unsafe {
        let idt_address = alloc_pages!(1).into();
        InterruptDescriptorTable::init(&mut IDT, idt_address);
        okprintln!("Initialized interrupt descriptor table");
        interrupt_handlers::init(IDT.assume_init_mut());
        okprintln!("Initialized interrupts handlers");
        CascadedPIC::init(&mut PIC);
        okprintln!("Initialized Programmable Interrupt Controller");
        let keyboard_buffer_address = alloc_pages!(1).into();
        Keyboard::init(
            &mut KEYBOARD,
            keyboard_buffer_address,
            NonZero::new(REGULAR_PAGE_SIZE).unwrap(),
        );
        okprintln!("Initialized Keyboard");
        interrupts::enable();
    }
    let mut pci_devices = pci::scan_pci();
    println!("Press ENTER to enumerate PCI devices!");
    let a = pci_devices.as_ptr() as usize;
    println!("pci_devices address: {:x}", a);

    loop {
        let c = unsafe { KEYBOARD.assume_init_mut().read_raw_scancode() };
        if let Some(e) = c
            && PS2ScanCode::from_scancode(e) == PS2ScanCode::Enter
        {
            break;
        }
    }

    for device in pci_devices.iter_mut() {
        // println!("{:#?}", unsafe { device.common.vendor_device });
        // println!("{:#?}", unsafe { device.common.header_type });
        // println!("{:#?}\n", unsafe { device.common.device_type });

        if device.common().device_type.is_ahci() {
            let a = unsafe {
                PhysicalAddress::new_unchecked(
                    device.general_device.bar5.address(),
                )
            };

            let aligned = a.align_down(REGULAR_PAGE_ALIGNMENT);
            let hba = HBAMemoryRegisters::new(aligned).unwrap();
            let active = hba.probe();
            let controller = hba.map_device(0);
            println!(
                "{:x?}",
                controller.port_commands as *const _ as usize
            )
        }
    }
    loop {
        unsafe {
            print!("{}", KEYBOARD.assume_init_mut().read_char() ; color = ColorCode::new(Color::Green, Color::Black));
        }
    }
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        interrupts::disable();
    }
    eprintln!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}
