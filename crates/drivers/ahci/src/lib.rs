#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(ascii_char)]
#![allow(static_mut_refs)]

pub mod fis;
pub mod hba;

use common::enums::CascadedPicInterruptLine;
pub use fis::*;
pub use hba::*;
use x86::structures::interrupt_descriptor_table::InterruptStackFrame;

use x86::pic8259::PIC;

pub extern "x86-interrupt" fn ahci_interrupt(
    _stack_frame: InterruptStackFrame,
) {
    unsafe { PIC.end_of_interrupt(CascadedPicInterruptLine::Ahci) };
}
