use common::enums::CascadedPicInterruptLine;
use x86::{
    instructions::interrupts,
    structures::interrupt_descriptor_table::InterruptStackFrame,
};

use crate::{PIC, WRITER};

pub extern "x86-interrupt" fn timer_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe {
        interrupts::disable();
    }
    if let Some(mut writer) = WRITER.try_lock() {
        writer.inner.update();
    }
    PIC.lock().end_of_interrupt(CascadedPicInterruptLine::Timer);
    unsafe {
        interrupts::enable();
    }
}
