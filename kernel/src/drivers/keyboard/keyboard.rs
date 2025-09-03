use core::{mem::MaybeUninit, num::NonZero};

use common::{address_types::VirtualAddress, enums::PS2ScanCode, flag, ring_buffer::RingBuffer};

pub struct KeyboardFlags(u8);

impl KeyboardFlags {
    pub fn default() -> Self {
        Self(0)
    }

    flag!(lshift_pressed, 0);
    flag!(rshift_pressed, 1);
    flag!(lctrl_pressed, 2);
    flag!(superkey_pressed, 3);
    flag!(capslock_pressed, 4);
}

pub struct Keyboard {
    pub(super) buffer: RingBuffer<u8>,
    pub(super) flags: KeyboardFlags,
}

impl Keyboard {
    pub fn init(uninit: &mut MaybeUninit<Self>, buffer: VirtualAddress, length: NonZero<usize>) {
        uninit.write(Keyboard {
            buffer: RingBuffer::new(buffer, length),
            flags: KeyboardFlags::default(),
        });
    }

    pub fn read_raw_scancode(&mut self) -> Option<u8> {
        self.buffer.read()
    }

    /// TODO change in the future to just return the relevant asscii code and not a long str
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
