use crate::structures::interrupt_descriptor_table::InterruptStackFrame;
use common::enums::CascadedPicInterruptLine;

use crate::pic8259::PIC;

pub extern "x86-interrupt" fn timer_handler(
    _stack_frame: InterruptStackFrame,
) {
    // print!(".");
    unsafe {
        PIC.end_of_interrupt(CascadedPicInterruptLine::Timer);
    }
}
