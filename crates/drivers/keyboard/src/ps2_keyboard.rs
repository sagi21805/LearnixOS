use core::{mem::MaybeUninit, ptr::NonNull};

use common::{enums::PS2ScanCode, ring_buffer::RingBuffer};

use macros::bitfields;

#[bitfields]
pub struct KeyboardFlags {
    pub lshift_pressed: B1,
    pub rshift_pressed: B1,
    pub lctrl_pressed: B1,
    pub superkey_pressed: B1,
    pub capslock_pressed: B1,
}

pub struct Keyboard {
    pub buffer: RingBuffer<u8>,
    pub flags: KeyboardFlags,
}

impl Keyboard {
    pub fn init(uninit: &mut MaybeUninit<Self>, buffer: NonNull<[u8]>) {
        uninit.write(Keyboard {
            buffer: RingBuffer::new(buffer),
            flags: KeyboardFlags::default(),
        });
    }

    pub fn read_raw_scancode(&mut self) -> Option<u8> {
        self.buffer.read()
    }

    /// TODO change in the future to just return the
    /// relevant ascii code and not a long str
    pub fn read_char(&mut self) -> &'static str {
        let key = match self.read_raw_scancode() {
            Some(scancode) => PS2ScanCode::from_scancode(scancode),
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
}
