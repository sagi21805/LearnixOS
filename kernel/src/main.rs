#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
extern crate alloc;

use libk::{okprintln, println};
use x86::instructions::cpuid;

mod console;
mod interrupts;
mod memory;
mod panic;
mod timer;

/// The kernel's entry point.
///
/// # Safety
///
/// Only the second stage may jump here, in long mode with interrupts
/// disabled and the boot page tables and the kernel stack set up.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub unsafe extern "C" fn _start() -> ! {
    okprintln!("Entered Protected Mode");
    okprintln!("Enabled Paging");
    okprintln!("Entered Long Mode");
    // SAFETY: the second stage stored the memory map before jumping here.
    let memory = unsafe { memory::init() };

    // SAFETY: the second stage's GDT is still loaded.
    unsafe { interrupts::init() };

    console::init_keyboard(memory.slab);

    let pci_devices = pci::scan_pci(memory.slab);
    println!("{}", memory.mmap);

    for device in pci_devices {
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

    console::run()
}

// The AHCI bring-up experiment, which ran after the hardware report and
// before the input loop.
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
