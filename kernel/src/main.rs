#![no_std]
#![no_main]
#![feature(ptr_alignment_type)]
#![feature(abi_x86_interrupt)]
#![feature(const_default)]
#![feature(const_trait_impl)]
extern crate alloc;

use core::panic::PanicInfo;

use alloc::boxed::Box;
use buddy::BuddyAllocator;
use bump::BumpAllocator;
use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    constants::{
        MEMORY_MAP_LENGTH, MEMORY_MAP_OFFSET, MiB, PARSED_MEMORY_MAP,
        PHYSICAL_MEMORY_OFFSET, REGULAR_PAGE_SIZE,
    },
    enums::{PS2ScanCode, PageSize},
    late_init::LateInit,
};
use keyboard::ps2_keyboard::Keyboard;
use page::{Page, arena::PageMap};
use vga_display::{
    SCREEN,
    advanced_writer::AdvancedWriter,
    eprintln,
    generic_writer::{GenericWriter, Writer},
    okprintln, vga_init,
    writer::SimpleWriter,
};
use x86::{
    instructions::interrupts::{self, hlt},
    memory_map::{MemoryMap, MemoryRegion, MemoryRegionExtended},
    pic8259::CascadedPIC,
    structures::interrupt_descriptor_table::InterruptDescriptorTable,
};

use libk::{
    alloc::{GlobalAllocator, VirtualAddressMapping},
    print, println,
};

use sync::{mutex::SpinMutex, spsc::SpscRingBuffer};

use crate::interrupt_handlers::InterruptDescriptorTableExt;

mod interrupt_handlers;
mod timer;

static MMAP: LateInit<MemoryMap> = LateInit::uninit();

#[unsafe(no_mangle)]
static PIC: SpinMutex<CascadedPIC> =
    SpinMutex::new(CascadedPIC::default());

static IDT: LateInit<SpinMutex<Box<InterruptDescriptorTable>>> =
    LateInit::uninit();

#[unsafe(no_mangle)]
static KEYBOARD: LateInit<Keyboard> = LateInit::uninit();

static KEYBOARD_BUFFER: LateInit<SpscRingBuffer<u8>> = LateInit::uninit();

#[unsafe(no_mangle)]
static SIMPLE_WRITER: SpinMutex<SimpleWriter<80, 25>> =
    unsafe { SpinMutex::new_locked(SimpleWriter::default()) };

static ADVANCED_WRITER: SpinMutex<LateInit<AdvancedWriter<80, 25>>> =
    unsafe { SpinMutex::new_locked(LateInit::uninit()) };

#[unsafe(no_mangle)]
static WRITER: SpinMutex<Writer<'static>> =
    SpinMutex::new(Writer::new(unsafe { SIMPLE_WRITER.leak() }));

pub static BUMP_ALLOCATOR: LateInit<BumpAllocator> = LateInit::uninit();

#[global_allocator]
static mut GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::uninit();

pub static BUDDY_ALLOCATOR: LateInit<BuddyAllocator<PageMap, Page>> =
    LateInit::uninit();

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _start() -> ! {
    vga_init();

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
        MMAP.init(MemoryMap::parse_map(raw, buf).unwrap());

        BUMP_ALLOCATOR.init(BumpAllocator::new(MMAP.assume_init_ref()));

        #[allow(static_mut_refs)]
        GLOBAL_ALLOCATOR.set(BUMP_ALLOCATOR.assume_init_ref());
    }

    BUDDY_ALLOCATOR.init(BuddyAllocator::<PageMap, Page>::new(
        MMAP.assume_init_ref(),
    ));

    BUDDY_ALLOCATOR.initialize(
        &*BUMP_ALLOCATOR.allocations.lock(),
        MMAP.assume_init_ref(),
    );

    #[allow(static_mut_refs)]
    GLOBAL_ALLOCATOR.set(BUDDY_ALLOCATOR.assume_init_ref());

    okprintln!("Initialized Buddy Allocator");
    unsafe {
        interrupts::disable();
        InterruptDescriptorTable::init(&IDT);
        okprintln!("Initialized interrupt descriptor table");
        IDT.lock().init_handlers();
        okprintln!("Initialized interrupts handlers");
        PIC.lock().init();
        okprintln!("Initialized Programmable Interrupt Controller");
        let buffer = Box::new([0u8; 4096]);
        KEYBOARD_BUFFER.init(SpscRingBuffer::new(buffer));

        KEYBOARD.init(Keyboard::new(&KEYBOARD_BUFFER));
        okprintln!("Initialized Keyboard");
        interrupts::enable();
    }
    let w = ADVANCED_WRITER.leak();
    w.init(AdvancedWriter::default());
    WRITER.lock().set_writer(w.assume_init_mut());
    okprintln!("Set advanced writer");
    // Wait for the next update.
    unsafe {
        hlt();
    }

    println!("{}", MMAP.assume_init_ref());
    BUDDY_ALLOCATOR.print_allocated_regions();
    // unsafe { SLAB_ALLOCATOR.init() }
    // okprintln!("Initialized slab allocator");
    // panic!("")
    // let mut pci_devices = pci::scan_pci();
    // println!("Press ENTER to enumerate PCI devices!");
    // let a = pci_devices.as_ptr() as usize;
    // println!("pci_devices address: {:x}", a);

    // loop {
    //     let c = unsafe {
    // KEYBOARD.assume_init_mut().read_raw_scancode() };     if
    // let Some(e) = c         && PS2ScanCode::from_scancode(e)
    // == PS2ScanCode::Enter     {
    //         break;
    //     }
    // }

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
    loop {
        let input = KEYBOARD.read_char();
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

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        SCREEN.force_unlock();
        SIMPLE_WRITER.force_unlock();
        // ADVANCED_WRITER.force_unlock();
        // WRITER.force_unlock();
    };
    // eprintln!("{}", _info ; color =
    // ColorCode::new().foreground(Color::Yellow).
    // background(Color::Black));
    SIMPLE_WRITER
        .lock()
        .panic_message(format_args!("{}", _info));
    loop {
        // let input = KEYBOARD.read_char();
        // match input {
        //     Err(key) => match key {
        //         PS2ScanCode::UpArrow =>
        // WRITER.lock().inner.scroll_up(1),
        //         PS2ScanCode::DownArrow => {
        //             WRITER.lock().inner.scroll_down(1)
        //         }
        //         _ => {}
        //     },
        //     Ok(_) => {}
        // }
    }
}
