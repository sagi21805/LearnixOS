use cpu_utils::structures::interrupt_descriptor_table::InterruptStackFrame;

use crate::println;

#[unsafe(no_mangle)] // keep the symbol name
#[inline(never)] // prevent inlining/removal due to optimization
pub extern "x86-interrupt" fn default_handler(_stack_frame: InterruptStackFrame) {
    println!("{:x?}", _stack_frame);
}
