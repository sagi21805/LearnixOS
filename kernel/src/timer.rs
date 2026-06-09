use common::enums::CascadedPicInterruptLine;
use x86::structures::interrupt_descriptor_table::InterruptStackFrame;

use crate::{PIC, WRITER};

pub extern "x86-interrupt" fn timer_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe {
        WRITER.inner.update();
        PIC.end_of_interrupt(CascadedPicInterruptLine::Timer);
    }
}
