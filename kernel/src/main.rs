#![no_std]
#![no_main]
#![feature(ptr_alignment_type)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(const_default)]
#![feature(const_trait_impl)]
#![allow(static_mut_refs)]
extern crate alloc;

use core::panic::PanicInfo;

use alloc::boxed::Box;
use bump::BumpAllocator;
use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    constants::{
        MEMORY_MAP_LENGTH, MEMORY_MAP_OFFSET, MiB, PARSED_MEMORY_MAP,
        PHYSICAL_MEMORY_OFFSET, REGULAR_PAGE_SIZE,
    },
    enums::{PS2ScanCode, PageSize},
    late_init::LateInit,
    ring_buffer::RingBuffer,
};
use keyboard::ps2_keyboard::Keyboard;
use vga_display::{
    advanced_writer::AdvancedWriter, eprintln, generic_writer::Writer,
    okprintln, screen_char::ScreenChar, writer::SimpleWriter,
};
use x86::{
    instructions::interrupts::{self, hlt},
    memory_map::{MemoryMap, MemoryRegion, MemoryRegionExtended},
    pic8259::CascadedPIC,
    structures::interrupt_descriptor_table::InterruptDescriptorTable,
};

use libk::{
    alloc::{BUMP_ALLOCATOR, GlobalAllocator, VirtualAddressExt},
    print, println,
};

mod interrupt_handlers;
mod timer;

static mut MMAP: LateInit<MemoryMap> = LateInit::uninit();
#[unsafe(no_mangle)]
static mut PIC: CascadedPIC = CascadedPIC::default();
static mut IDT: LateInit<Box<InterruptDescriptorTable>> =
    LateInit::uninit();

#[unsafe(no_mangle)]
static mut KEYBOARD: LateInit<Keyboard> = LateInit::uninit();
#[unsafe(no_mangle)]
static mut SIMPLE_WRITER: LateInit<SimpleWriter<80, 25>> =
    LateInit::new(SimpleWriter::default());
#[unsafe(no_mangle)]
static mut WRITER: Writer<'static> =
    Writer::new(unsafe { SIMPLE_WRITER.assume_init_mut() });
#[global_allocator]
static mut GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::uninit();
static mut ADVANCED_WRITER: LateInit<AdvancedWriter<80, 25>> =
    LateInit::uninit();

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _start() -> ! {
    okprintln!("Entered Protected Mode");
    okprintln!("Enabled Paging");
    okprintln!("Entered Long Mode");

    let len = unsafe { *(MEMORY_MAP_LENGTH as *const u32) as usize };
    let raw = unsafe {
        core::slice::from_raw_parts_mut(
            MEMORY_MAP_OFFSET as *mut MemoryRegionExtended,
            len,
        )
    };
    let buf = unsafe {
        core::slice::from_raw_parts_mut(
            PARSED_MEMORY_MAP as *mut MemoryRegion,
            REGULAR_PAGE_SIZE / size_of::<MemoryRegion>(),
        )
    };

    unsafe {
        let _ = MMAP.write(MemoryMap::parse_map(raw, buf).unwrap());

        let _ = BUMP_ALLOCATOR
            .write(BumpAllocator::new(MMAP.assume_init_ref()));

        GLOBAL_ALLOCATOR.init(BUMP_ALLOCATOR.assume_init_ref());
        let v = VirtualAddress::new_unchecked(
            PHYSICAL_MEMORY_OFFSET + 6 * MiB,
        );
        let p = PhysicalAddress::new_unchecked(6 * MiB);

        println!("Virtual address: {:x?} is mapped: {}", v, v.is_mapped());

        let succ = v.map(p, None, PageSize::Big);

        println!("Map succeeded: {:?}", succ);

        println!("Virtual address: {:x?} is mapped: {}", v, v.is_mapped());
        println!("Map succeeded: {:?}", succ);
    }

    unsafe {
        interrupts::disable();
        InterruptDescriptorTable::init(&mut IDT);
        okprintln!("Initialized interrupt descriptor table");
        interrupt_handlers::init(IDT.as_mut());
        okprintln!("Initialized interrupts handlers");
        CascadedPIC::init(&mut PIC);
        okprintln!("Initialized Programmable Interrupt Controller");
        Keyboard::init(&mut KEYBOARD);
        okprintln!("Initialized Keyboard");
        interrupts::enable();
    }
    ADVANCED_WRITER.write(AdvancedWriter::default());
    WRITER.set_writer(ADVANCED_WRITER.assume_init_mut());
    // unsafe { SLAB_ALLOCATOR.init() }
    // okprintln!("Initialized slab allocator");
    // ::core::arch::asm!("int 3");
    // panic!("")
    // let mut pci_devices = pci::scan_pci();
    // println!("Press ENTER to enumerate PCI devices!");
    // let a = pci_devices.as_ptr() as usize;
    // println!("pci_devices address: {:x}", a);

    // loop {
    //     let c = unsafe { KEYBOARD.assume_init_mut().read_raw_scancode()
    // };     if let Some(e) = c
    //         && PS2ScanCode::from_scancode(e) == PS2ScanCode::Enter
    //     {
    //         break;
    //     }
    // }

    // unsafe { PIC.enable_irq(CascadedPicInterruptLine::Ahci) };
    // for device in pci_devices.iter_mut() {
    //     // println!("{:#?}", unsafe { device.common.vendor_device });
    //     // println!("{:#?}", unsafe { device.common.header_type });
    //     // println!("{:#?}\n", unsafe { device.common.device_type });

    //     if device.header.common().device_type.is_ahci() {
    //         let a = unsafe {
    //             PhysicalAddress::new_unchecked(
    //                 device.header.general_device.bar5.address(),
    //             )
    //         };

    //         println!(
    //             "Bus Master: {}, Interrupts Disable {}, I/O Space: {}, \
    //              Memory Space: {}",
    //             device.header.common().command.is_bus_master(),
    //             device.header.common().command.is_interrupt_disable(),
    //             device.header.common().command.is_io_space(),
    //             device.header.common().command.is_memory_space()
    //         );

    //         println!(
    //             "Interrupt Line: {}, Interrupt Pin: {}",
    //             unsafe { device.header.general_device.interrupt_line },
    //             unsafe { device.header.general_device.interrupt_pin }
    //         );

    //         let aligned = a.align_down(REGULAR_PAGE_ALIGNMENT);
    //         let hba = HBAMemoryRegisters::new(aligned).unwrap();
    //         let _ = hba.probe_init();
    //         let p = &mut hba.ports[0];

    //         let buf =
    //             unsafe { alloc_pages!(1) as *mut IdentityPacketData };

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

    loop {
        let scancode = KEYBOARD.read_raw_scancode();
        if let Some(scancode) = scancode {
            match scancode {
                PS2ScanCode::Keypad8 => WRITER.inner.scroll_down(1),
                PS2ScanCode::Keypad2 => WRITER.inner.scroll_up(1),
                _ => print!("{}", scancode),
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
    eprintln!("{}", _info ; color = ColorCode::new().foreground(Color::Yellow).background(Color::Black));
    loop {}
}
