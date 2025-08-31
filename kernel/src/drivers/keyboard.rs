use core::mem::MaybeUninit;

use crate::drivers::pic8259::PIC;
use common::{
    address_types::VirtualAddress,
    enums::{CascadedPicInterruptLine, Port},
    ring_buffer::RingBuffer,
};
use cpu_utils::{
    instructions::port::PortExt, structures::interrupt_descriptor_table::InterruptStackFrame,
};

pub static mut KEYBOARD_BUFFER: MaybeUninit<RingBuffer<u8>> = MaybeUninit::uninit();

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let scan_code = Port::KeyboardData.inb();
        KEYBOARD_BUFFER.assume_init_mut().write(scan_code);

        PIC.assume_init_mut()
            .end_of_interrupt(CascadedPicInterruptLine::Keyboard);
    }
}

pub fn init(
    uninit: &'static mut MaybeUninit<RingBuffer<u8>>,
    buffer: VirtualAddress,
    length: usize,
) {
    uninit.write(RingBuffer::new(buffer, length));
}
