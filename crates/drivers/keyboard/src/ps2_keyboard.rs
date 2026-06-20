extern crate alloc;

use common::enums::PS2ScanCode;

use macros::bitfields;
use sync::spsc::{Consumer, Producer, SpscRingBuffer};
use x86::instructions::interrupts::hlt;

#[bitfields]
pub struct KeyboardFlags {
    pub lshift_pressed: B1,
    pub rshift_pressed: B1,
    pub lctrl_pressed: B1,
    pub superkey_pressed: B1,
    pub capslock_pressed: B1,
}

pub struct Keyboard {
    pub(crate) producer: Producer<'static, u8>,
    pub(crate) flags: KeyboardFlags,
    consumer: Consumer<'static, u8>,
}

impl Keyboard {
    pub fn new(spsc: &'static SpscRingBuffer<u8>) -> Self {
        let Some((producer, consumer)) = spsc.try_utilize() else {
            todo!();
        };
        Keyboard {
            producer,
            consumer,
            flags: KeyboardFlags::new(),
        }
    }

    pub fn read_raw_scancode(&self) -> Option<PS2ScanCode> {
        // unsafe { hlt() };
        PS2ScanCode::try_from(self.consumer.pop()?).ok()
    }

    /// TODO change in the future to just return the
    /// relevant ascii code and not a long str
    pub fn read_char(&mut self) -> &'static str {
        let key = match self.read_raw_scancode() {
            Some(scancode) => PS2ScanCode::from(scancode),
            None => return "",
        };
        if self.flags.is_lshift_pressed()
            || self.flags.is_rshift_pressed()
            || self.flags.is_capslock_pressed()
        {
            key.to_str_shifted()
        } else {
            key.to_str()
        }
    }

    pub unsafe fn consumer(&self) -> &Consumer<'static, u8> {
        &self.consumer
    }
}
