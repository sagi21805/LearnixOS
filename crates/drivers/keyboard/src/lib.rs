#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

use core::cell::OnceCell;

use common::enums::{CascadedPicInterruptLine, Port};
use sync::mutex::SpinMutex;
use x86::{
    instructions::{interrupts, port::PortExt},
    pic8259::CascadedPIC,
    structures::interrupt_descriptor_table::InterruptStackFrame,
};

use crate::ps2_keyboard::Keyboard;

pub mod ps2_keyboard;

unsafe extern "Rust" {
    unsafe static KEYBOARD: SpinMutex<OnceCell<Keyboard>>;
    unsafe static PIC: SpinMutex<CascadedPIC>;
}

pub extern "x86-interrupt" fn keyboard_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe { interrupts::disable() };
    let mut locked = unsafe { KEYBOARD.lock() };
    let Some(keyboard) = locked.get_mut() else {
        return;
    };
    unsafe {
        keyboard.handle_scan_code(Port::KeyboardData.inb());
        PIC.lock()
            .end_of_interrupt(CascadedPicInterruptLine::Keyboard);
        interrupts::enable();
    }
}
