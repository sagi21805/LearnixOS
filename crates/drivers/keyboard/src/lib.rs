#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

use common::{
    enums::{CascadedPicInterruptLine, PS2ScanCode, Port},
    late_init::LateInit,
};
use x86::{
    instructions::port::PortExt, pic8259::PIC,
    structures::interrupt_descriptor_table::InterruptStackFrame,
};

use crate::ps2_keyboard::Keyboard;

pub mod ps2_keyboard;

pub static mut KEYBOARD: LateInit<Keyboard> = LateInit::uninit();

#[allow(static_mut_refs)]
pub extern "x86-interrupt" fn keyboard_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe {
        let scan_code = Port::KeyboardData.inb();
        KEYBOARD.buffer.write(scan_code);
        match PS2ScanCode::from_scancode(scan_code) {
            PS2ScanCode::LeftShift => {
                KEYBOARD.flags.set_lshift_pressed(true)
            }
            PS2ScanCode::ReleasedLeftShift => {
                KEYBOARD.flags.set_lshift_pressed(false)
            }
            PS2ScanCode::RightShift => {
                KEYBOARD.flags.set_rshift_pressed(true)
            }
            PS2ScanCode::ReleasedRightShift => {
                KEYBOARD.flags.set_rshift_pressed(false)
            }
            PS2ScanCode::LeftCtrl => {
                KEYBOARD.flags.set_lctrl_pressed(true)
            }
            PS2ScanCode::ReleasedLeftCtrl => {
                KEYBOARD.flags.set_lctrl_pressed(false)
            }
            PS2ScanCode::SuperKey => {
                KEYBOARD.flags.set_superkey_pressed(true)
            }
            PS2ScanCode::ReleasedSuperKey => {
                KEYBOARD.flags.set_superkey_pressed(false)
            }
            PS2ScanCode::CapsLock => {
                KEYBOARD.flags.set_capslock_pressed(
                    KEYBOARD.flags.is_capslock_pressed() ^ true,
                );
            }
            _ => {}
        }
        PIC.end_of_interrupt(CascadedPicInterruptLine::Keyboard);
    }
}
