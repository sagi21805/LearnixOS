use alloc::boxed::Box;
use common::enums::interrupts::GateType;
use keyboard::keyboard_handler;
use sync::once::Once;
use vga_display::okprintln;
use x86::{
    instructions::interrupts::disable,
    pic8259::PIC,
    structures::{
        interrupt_descriptor_table::InterruptDescriptorTable,
        segments::TSS,
    },
};

pub use x86::instructions::interrupts::enable;

use crate::{
    interrupts::exceptions::ExceptionHandlers, timer::timer_handler,
};

mod exceptions;

static IDT: Once<Box<InterruptDescriptorTable>> = Once::uninit();

/// Loads the TSS and the IDT and sets up the PIC, leaving interrupts
/// disabled so the caller chooses when the first one fires.
///
/// # Safety
///
/// Only `_start` may call this, after the global allocator is set because
/// the table lives in a `Box`. The second stage's GDT must still be
/// loaded, because the TSS installs itself there.
pub unsafe fn init() {
    unsafe { disable() };

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
