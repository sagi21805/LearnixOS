use common::enums::CascadedPicInterruptLine;
use x86::{
    pic8259::PIC,
    structures::interrupt_descriptor_table::InterruptStackFrame,
};

pub extern "x86-interrupt" fn timer_handler(
    _stack_frame: InterruptStackFrame,
) {
    PIC.lock().end_of_interrupt(CascadedPicInterruptLine::Timer);
}
