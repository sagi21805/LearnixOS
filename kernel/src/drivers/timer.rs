use crate::print;
use cpu_utils::structures::interrupt_descriptor_table::InterruptStackFrame;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!("{:?}", _stack_frame);
}
