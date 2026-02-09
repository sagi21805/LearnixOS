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
#![feature(slice_ptr_get)]
#![feature(core_intrinsics)]
#![feature(explicit_tail_calls)]
#![feature(specialization)]
#![deny(clippy::all)]
mod drivers;
mod memory;
use core::{num::NonZero, panic::PanicInfo};

use crate::{
    drivers::{
        interrupt_handlers,
        keyboard::{ps2_keyboard::Keyboard, KEYBOARD},
        pic8259::{CascadedPIC, PIC},
        vga_display::color_code::ColorCode,
    },
    memory::{
        allocators::{
            buddy::BUDDY_ALLOCATOR, extensions::PageTableExt,
            slab::SLAB_ALLOCATOR,
        },
        memory_map::{parse_map, MemoryMap},
        page::{map::PageMap, PAGES},
    },
};

use common::{constants::REGULAR_PAGE_SIZE, enums::Color};
use cpu_utils::{
    instructions::interrupts::{self},
    structures::{
        interrupt_descriptor_table::{InterruptDescriptorTable, IDT},
        paging::PageTable,
    },
};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _start() -> ! {
    okprintln!("Entered Protected Mode");
    okprintln!("Enabled Paging");
    okprintln!("Entered Long Mode");
    parse_map();
    okprintln!("Obtained Memory Map");
    println!("{}", MemoryMap(parsed_memory_map!()));

    PageMap::init(unsafe { &mut PAGES }, MemoryMap(parsed_memory_map!()));
    unsafe { BUDDY_ALLOCATOR.init(MemoryMap(parsed_memory_map!()), 0) };

    let last = MemoryMap(parsed_memory_map!()).last().unwrap();

    unsafe {
        PageTable::current_table().as_mut().map_physical_memory(
            (last.base_address + last.length) as usize,
        );
    }
    okprintln!("Initialized buddy allocator");
    unsafe {
        InterruptDescriptorTable::init(
            &mut IDT,
            alloc_pages!(1).translate(),
        );
        okprintln!("Initialized interrupt descriptor table");
        interrupt_handlers::init(IDT.assume_init_mut());
        okprintln!("Initialized interrupts handlers");
        CascadedPIC::init(&mut PIC);

        okprintln!("Initialized Programmable Interrupt Controller");
        let keyboard_buffer_address: common::address_types::VirtualAddress = alloc_pages!(1).translate();
        Keyboard::init(
            &mut KEYBOARD,
            keyboard_buffer_address,
            NonZero::new(REGULAR_PAGE_SIZE).unwrap(),
        );
        okprintln!("Initialized Keyboard");
        interrupts::enable();
    }

    unsafe { SLAB_ALLOCATOR.init() }
    okprintln!("Initialized slab allocator");

    panic!("")
    let mut pci_devices = pci::scan_pci();
    println!("Press ENTER to enumerate PCI devices!");
    let a = pci_devices.as_ptr() as usize;
    println!("pci_devices address: {:x}", a);

    loop {
        let c = unsafe { KEYBOARD.assume_init_mut().read_raw_scancode()
    };     if let Some(e) = c
            && PS2ScanCode::from_scancode(e) == PS2ScanCode::Enter
        {
            break;
        }
    }

    unsafe { PIC.enable_irq(CascadedPicInterruptLine::Ahci) };
    for device in pci_devices.iter_mut() {
        // println!("{:#?}", unsafe { device.common.vendor_device });
        // println!("{:#?}", unsafe { device.common.header_type });
        // println!("{:#?}\n", unsafe { device.common.device_type });

        if device.header.common().device_type.is_ahci() {
            let a = unsafe {
                PhysicalAddress::new_unchecked(
                    device.header.general_device.bar5.address(),
                )
            };

            println!(
                "Bus Master: {}, Interrupts Disable {}, I/O Space: {}, \
                 Memory Space: {}",
                device.header.common().command.is_bus_master(),
                device.header.common().command.is_interrupt_disable(),
                device.header.common().command.is_io_space(),
                device.header.common().command.is_memory_space()
            );

            println!(
                "Interrupt Line: {}, Interrupt Pin: {}",
                unsafe { device.header.general_device.interrupt_line },
                unsafe { device.header.general_device.interrupt_pin }
            );

            let aligned = a.align_down(REGULAR_PAGE_ALIGNMENT);
            let hba = HBAMemoryRegisters::new(aligned).unwrap();
            let _ = hba.probe_init();
            let p = &mut hba.ports[0];

            let buf =
                unsafe { alloc_pages!(1) as *mut IdentityPacketData };

            p.identity_packet(buf);

            let id = unsafe {
                core::ptr::read_volatile(
                    (buf as usize + PHYSICAL_MEMORY_OFFSET)
                        as *mut IdentityPacketData,
                )
            };

            println!("{:?}", id);

            println!("Cylinders: {}", id.cylinders);
            println!("Heads: {}", id.heads);
            println!("Sectors: {}", id.sectors);

            println!("Serial: {:?}", &id.serial_number);
            println!("Model: {:?}", &id.model_num);
            println!("Firmware: {:?}", &id.firmware_rev);
        }
    }

    loop {
        unsafe {
            print!("{}", KEYBOARD.assume_init_mut().read_char() ; color
    = ColorCode::new(Color::Green, Color::Black));
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
