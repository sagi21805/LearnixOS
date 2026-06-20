use common::enums::CascadedPicInterruptLine;
use vga_display::SCREEN_LOCK;
use x86::structures::interrupt_descriptor_table::InterruptStackFrame;

use crate::{PIC, WRITER};

pub extern "x86-interrupt" fn timer_handler(
    _stack_frame: InterruptStackFrame,
) {
    WRITER.inner.lock().update();
    PIC.lock().end_of_interrupt(CascadedPicInterruptLine::Timer);
}
