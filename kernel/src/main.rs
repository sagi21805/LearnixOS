#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
extern crate alloc;

use alloc::boxed::Box;
use common::enums::PS2ScanCode;
use keyboard::{KEYBOARD, KEYBOARD_BUFFER, ps2_keyboard::Keyboard};
use vga_display::{
    ADVANCED_WRITER, WRITER, advanced_writer::AdvancedWriter, okprintln,
    print, println, vga_init,
};
use x86::{
    instructions::{cpuid, interrupts as cpu_interrupts},
    pic8259::PIC,
    structures::{
        interrupt_descriptor_table::InterruptDescriptorTable,
        segments::TSS,
    },
};

use sync::{mutex::SpinMutex, once::Once, spsc::SpscRingBuffer};

use crate::interrupts::exceptions::InterruptDescriptorTableExt;

mod interrupts;
mod memory;
mod panic;
mod timer;

static IDT: Once<Box<InterruptDescriptorTable>> = Once::uninit();

/// The kernel's entry point.
///
/// # Safety
///
/// Only the second stage may jump here, in long mode with the boot page
/// tables and the kernel stack set up.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub unsafe extern "C" fn _start() -> ! {
    vga_init();

    okprintln!("Entered Protected Mode");
    okprintln!("Enabled Paging");
    okprintln!("Entered Long Mode");
    // SAFETY: the second stage stored the memory map before jumping here.
    let memory = unsafe { memory::init() };

    unsafe {
        cpu_interrupts::disable();
        TSS.load();
        okprintln!("Loaded task state segment");
        let mut table = Box::new(InterruptDescriptorTable::default());
        table.init_handlers();
        IDT.init(table).load();
        okprintln!("Initialized interrupt descriptor table");
        PIC.lock().init();
        okprintln!("Initialized Programmable Interrupt Controller");
        let buffer: Box<[u8], _> = Box::new_in([0u8; 4096], memory.slab);
        let buffer_address = buffer.as_ptr();
        let keyboard_buffer =
            KEYBOARD_BUFFER.init(SpscRingBuffer::new(buffer));

        KEYBOARD.init(SpinMutex::new(Keyboard::new(keyboard_buffer)));
        okprintln!("Initialized Keyboard at {:p}", buffer_address);
        cpu_interrupts::enable();
    }
    // SAFETY: this is the only `leak` of the writer, and it stays locked.
    let w = unsafe {
        ADVANCED_WRITER
            .init(SpinMutex::new_locked(AdvancedWriter::default()))
            .leak()
    };
    WRITER.lock().set_writer(w);
    okprintln!("Set advanced writer");
    // Wait for the next update.
    unsafe {
        cpu_interrupts::hlt();
    }

    println!("{}", memory.mmap);

    let pci_devices = pci::scan_pci(memory.slab);
    for device in &pci_devices {
        println!("{}", device);
    }
    println!("");

    let vendor = cpuid::get_vendor_string();
    // SAFETY: the vendor string is ASCII, so it is valid UTF-8.
    let vendor = unsafe { str::from_utf8_unchecked(&vendor) };
    println!("CPU vendor: {}", vendor);
    if let Some(brand) = cpuid::get_brand_string() {
        println!("CPU brand: {}", brand.as_str());
    }

    loop {
        // SAFETY: the keyboard handler takes the same lock, so it must
        // not run while this loop holds it.
        unsafe { cpu_interrupts::disable() };
        let input = KEYBOARD.lock().read_char();
        unsafe { cpu_interrupts::enable() };
        match input {
            Ok(str) => print!("{}", str),
            Err(key) => match key {
                PS2ScanCode::UpArrow => WRITER.lock().inner.scroll_up(1),
                PS2ScanCode::DownArrow => {
                    WRITER.lock().inner.scroll_down(1)
                }
                PS2ScanCode::LeftArrow => {
                    WRITER.lock().inner.scroll_up(24)
                }
                PS2ScanCode::RightArrow => {
                    WRITER.lock().inner.scroll_down(24)
                }
                _ => {}
            },
        }
    }
}

// The AHCI bring-up I experimented with, which ran after the hardware
// report and before the input loop.
//
// unsafe { PIC.enable_irq(CascadedPicInterruptLine::Ahci) };
// for device in pci_devices.iter_mut() {
//     // println!("{:#?}", unsafe { device.common.vendor_device
// });     // println!("{:#?}", unsafe {
// device.common.header_type });     // println!("{:#?}\n",
// unsafe { device.common.device_type });

//     if device.header.common().device_type.is_ahci() {
//         let a = unsafe {
//             PhysicalAddress::new_unchecked(
//                 device.header.general_device.bar5.address(),
//             )
//         };

//         println!(
//             "Bus Master: {}, Interrupts Disable {}, I/O Space:
// {}, \              Memory Space: {}",
//             device.header.common().command.is_bus_master(),
//
// device.header.common().command.is_interrupt_disable(),
//             device.header.common().command.is_io_space(),
//             device.header.common().command.is_memory_space()
//         );

//         println!(
//             "Interrupt Line: {}, Interrupt Pin: {}",
//             unsafe { device.header.general_device.interrupt_line
// },             unsafe {
// device.header.general_device.interrupt_pin }         );

//         let aligned = a.align_down(REGULAR_PAGE_ALIGNMENT);
//         let hba = HBAMemoryRegisters::new(aligned).unwrap();
//         let _ = hba.probe_init();
//         let p = &mut hba.ports[0];

//         let buf =
//             unsafe { alloc_pages!(1) as *mut IdentityPacketData
// };

//         p.identity_packet(buf);
//         let id = unsafe {
//             core::ptr::read_volatile(
//                 (buf as usize + PHYSICAL_MEMORY_OFFSET)
//                     as *mut IdentityPacketData,
//             )
//         };

//         println!("{:?}", id);

//         println!("Cylinders: {}", id.cylinders);
//         println!("Heads: {}", id.heads);
//         println!("Sectors: {}", id.sectors);

//         println!("Serial: {:?}", &id.serial_number);
//         println!("Model: {:?}", &id.model_num);
//         println!("Firmware: {:?}", &id.firmware_rev);
//     }
// }
