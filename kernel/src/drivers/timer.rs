use common::enums::CascadedPicInterruptLine;
use cpu_utils::structures::interrupt_descriptor_table::InterruptStackFrame;

use crate::drivers::pic8259::PIC;

pub extern "x86-interrupt" fn timer_handler(
    _stack_frame: InterruptStackFrame,
) {
    // print!(".");
    unsafe {
        PIC.assume_init_mut()
            .end_of_interrupt(CascadedPicInterruptLine::Timer);
    }
}
