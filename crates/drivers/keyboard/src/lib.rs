#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

use core::cell::OnceCell;

use common::enums::{CascadedPicInterruptLine, PS2ScanCode, Port};
use sync::mutex::SpinMutex;
use x86::{
    instructions::port::PortExt, pic8259::CascadedPIC,
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
    let mut locked = unsafe { KEYBOARD.lock() };
    let Some(keyboard) = locked.get_mut() else {
        return;
    };
    unsafe {
        let scan_code = Port::KeyboardData.inb();
        keyboard.producer.push(scan_code);
        match PS2ScanCode::from(scan_code) {
            PS2ScanCode::LeftShift => {
                keyboard.flags.set_lshift_pressed(true)
            }
            PS2ScanCode::ReleasedLeftShift => {
                keyboard.flags.set_lshift_pressed(false)
            }
            PS2ScanCode::RightShift => {
                keyboard.flags.set_rshift_pressed(true)
            }
            PS2ScanCode::ReleasedRightShift => {
                keyboard.flags.set_rshift_pressed(false)
            }
            PS2ScanCode::LeftCtrl => {
                keyboard.flags.set_lctrl_pressed(true)
            }
            PS2ScanCode::ReleasedLeftCtrl => {
                keyboard.flags.set_lctrl_pressed(false)
            }
            PS2ScanCode::SuperKey => {
                keyboard.flags.set_superkey_pressed(true)
            }
            PS2ScanCode::ReleasedSuperKey => {
                keyboard.flags.set_superkey_pressed(false)
            }
            PS2ScanCode::CapsLock => {
                keyboard.flags.set_capslock_pressed(
                    keyboard.flags.is_capslock_pressed() ^ true,
                );
            }
            _ => {}
        }
        PIC.lock()
            .end_of_interrupt(CascadedPicInterruptLine::Keyboard);
    }
}
