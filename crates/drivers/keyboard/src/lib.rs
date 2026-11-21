#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

use common::enums::{CascadedPicInterruptLine, Port};
use sync::once::Once;
use x86::{
    instructions::port::PortExt, interrupt_safe_mutex::InterruptSafeMutex,
    pic8259::PIC,
    structures::interrupt_descriptor_table::InterruptStackFrame,
};

use crate::{ps2_keyboard::Keyboard, waker_slot::WakerSlot};

pub mod ps2_keyboard;
pub mod waker_slot;

/// The PS/2 keyboard, which [`keyboard_handler`] feeds once it is set.
pub static KEYBOARD: Once<InterruptSafeMutex<Keyboard>> = Once::uninit();

/// The task that waits for [`KEYBOARD`]'s next scan code.
pub static KEYBOARD_WAKER: WakerSlot = WakerSlot::new();

pub extern "x86-interrupt" fn keyboard_handler(
    _stack_frame: InterruptStackFrame,
) {
    // The controller holds its next byte until this read, so it always
    // runs.
    let scan_code = unsafe { Port::KeyboardData.inb() };
    if let Some(keyboard) = KEYBOARD.get() {
        keyboard.lock().handle_scan_code(scan_code);
    }
    KEYBOARD_WAKER.wake();
    PIC.lock()
        .end_of_interrupt(CascadedPicInterruptLine::Keyboard);
}
