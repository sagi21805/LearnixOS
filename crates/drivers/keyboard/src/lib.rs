#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

use common::enums::{CascadedPicInterruptLine, Port};
use sync::{mutex::SpinMutex, once::Once, spsc::SpscRingBuffer};
use x86::{
    instructions::{interrupts, port::PortExt},
    pic8259::PIC,
    structures::interrupt_descriptor_table::InterruptStackFrame,
};

use crate::ps2_keyboard::Keyboard;

pub mod ps2_keyboard;

/// The PS/2 keyboard, which [`keyboard_handler`] feeds once it is set.
pub static KEYBOARD: Once<SpinMutex<Keyboard>> = Once::uninit();

/// The scan codes [`keyboard_handler`] queues for [`KEYBOARD`]'s reader.
pub static KEYBOARD_BUFFER: Once<SpscRingBuffer<u8>> = Once::uninit();

pub extern "x86-interrupt" fn keyboard_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe { interrupts::disable() };
    // The controller holds its next byte until this read, so it always
    // runs.
    let scan_code = unsafe { Port::KeyboardData.inb() };
    if let Some(keyboard) = KEYBOARD.get() {
        keyboard.lock().handle_scan_code(scan_code);
    }
    PIC.lock()
        .end_of_interrupt(CascadedPicInterruptLine::Keyboard);
    unsafe { interrupts::enable() };
}
