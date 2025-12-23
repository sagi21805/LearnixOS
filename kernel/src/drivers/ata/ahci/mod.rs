pub mod fis;
pub mod hba;

use cpu_utils::structures::interrupt_descriptor_table::InterruptStackFrame;
pub use fis::*;
pub use hba::*;

pub extern "x86-interrupt" fn ahci_interrupt(
    _stack_frame: InterruptStackFrame,
) {
    panic!("AHCI Interrupts!");
}
