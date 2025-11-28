use core::mem::MaybeUninit;

use common::enums::{CascadedPicInterruptLine, PS2ScanCode, Port};
use cpu_utils::{
    instructions::port::PortExt,
    structures::interrupt_descriptor_table::InterruptStackFrame,
};

use crate::drivers::{keyboard::ps2_keyboard::Keyboard, pic8259::PIC};

pub mod ps2_keyboard;

pub static mut KEYBOARD: MaybeUninit<Keyboard> = MaybeUninit::uninit();

pub extern "x86-interrupt" fn keyboard_handler(
    _stack_frame: InterruptStackFrame,
) {
    unsafe {
        let keyboard = KEYBOARD.assume_init_mut();
        let scan_code = Port::KeyboardData.inb();
        keyboard.buffer.write(scan_code);
        match PS2ScanCode::from_scancode(scan_code) {
            PS2ScanCode::LeftShift => keyboard.flags.set_lshift_pressed(),
            PS2ScanCode::ReleasedLeftShift => {
                keyboard.flags.unset_lshift_pressed()
            }
            PS2ScanCode::RightShift => keyboard.flags.set_rshift_pressed(),
            PS2ScanCode::ReleasedRightShift => {
                keyboard.flags.unset_rshift_pressed()
            }
            PS2ScanCode::LeftCtrl => keyboard.flags.set_lctrl_pressed(),
            PS2ScanCode::ReleasedLeftCtrl => {
                keyboard.flags.unset_lctrl_pressed()
            }
            PS2ScanCode::SuperKey => keyboard.flags.set_superkey_pressed(),
            PS2ScanCode::ReleasedSuperKey => {
                keyboard.flags.unset_superkey_pressed()
            }
            PS2ScanCode::CapsLock => {
                if keyboard.flags.is_capslock_pressed() {
                    keyboard.flags.unset_capslock_pressed();
                } else {
                    keyboard.flags.set_capslock_pressed();
                }
            }
            _ => {}
        }
        PIC.assume_init_mut()
            .end_of_interrupt(CascadedPicInterruptLine::Keyboard);
    }
}
