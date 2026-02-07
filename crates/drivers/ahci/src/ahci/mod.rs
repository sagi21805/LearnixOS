pub mod fis;
pub mod hba;

use common::enums::CascadedPicInterruptLine;
use cpu_utils::structures::interrupt_descriptor_table::InterruptStackFrame;
pub use fis::*;
pub use hba::*;

use crate::{drivers::pic8259::PIC, println};

pub extern "x86-interrupt" fn ahci_interrupt(
    _stack_frame: InterruptStackFrame,
) {
    println!("AHCI Interrupts!");
    unsafe { PIC.end_of_interrupt(CascadedPicInterruptLine::Ahci) };
}
