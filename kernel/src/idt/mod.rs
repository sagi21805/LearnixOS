use alloc::boxed::Box;
use common::enums::interrupts::GateType;
use keyboard::keyboard_handler;
use libk::okprintln;
use sync::once::Once;
use x86::{
    instructions::interrupts,
    pic8259::PIC,
    structures::{
        interrupt_descriptor_table::InterruptDescriptorTable,
        segments::TSS,
    },
};

use crate::{idt::exceptions::ExceptionHandlers, timer::timer_handler};

mod exceptions;

static IDT: Once<Box<InterruptDescriptorTable>> = Once::uninit();

/// Loads the TSS and the IDT, sets up the PIC and enables interrupts.
///
/// # Safety
///
/// The loaded GDT must be a `GlobalDescriptorTableLong` in writable
/// memory, because the TSS installs itself there.
pub unsafe fn init() {
    // SAFETY: the kernel is still initializing, so nothing relies on
    // interrupts being on.
    unsafe { interrupts::disable() };

    unsafe { TSS.load() };
    okprintln!("Loaded task state segment");

    let mut table = Box::new(InterruptDescriptorTable::default());
    table.set_exception_handlers();
    table.set_irq_handlers();
    // SAFETY: the loaded GDT holds kernel code at `Sections::KernelCode`.
    unsafe { IDT.init(table).load() };
    okprintln!("Initialized interrupt descriptor table");

    PIC.lock().init();
    okprintln!("Initialized Programmable Interrupt Controller");

    // SAFETY: the CPU has its exception and IRQ handlers now.
    unsafe { interrupts::enable() };
}

#[extend::ext]
impl InterruptDescriptorTable {
    /// Registers the handlers of the hardware interrupts in use.
    fn set_irq_handlers(&mut self) {
        self.timer.set_handler(timer_handler, GateType::Trap);
        self.keyboard.set_handler(keyboard_handler, GateType::Trap);
        // self.ahci.set_handler(ahci_interrupt, GateType::Trap);
    }
}
