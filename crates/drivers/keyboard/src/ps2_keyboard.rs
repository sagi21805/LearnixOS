extern crate alloc;

use common::enums::PS2ScanCode;

use bitfields::bitfields;
use sync::spsc::{Consumer, Producer, SpscRingBuffer};

#[bitfields]
pub struct KeyboardFlags {
    pub lshift_pressed: B1,
    pub rshift_pressed: B1,
    pub lctrl_pressed: B1,
    pub superkey_pressed: B1,
    pub capslock_pressed: B1,
}

pub struct Keyboard {
    producer: Producer<'static, u8>,
    flags: KeyboardFlags,
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

    /// Queues `scan_code` for [`Self::read_char`], and updates the
    /// modifier flags it changes.
    pub(crate) fn handle_scan_code(&mut self, scan_code: u8) {
        self.producer.push(scan_code);
        match PS2ScanCode::from(scan_code) {
            PS2ScanCode::LeftShift => {
                self.flags.set_lshift_pressed(true);
            }
            PS2ScanCode::ReleasedLeftShift => {
                self.flags.set_lshift_pressed(false)
            }
            PS2ScanCode::RightShift => self.flags.set_rshift_pressed(true),
            PS2ScanCode::ReleasedRightShift => {
                self.flags.set_rshift_pressed(false)
            }
            PS2ScanCode::LeftCtrl => self.flags.set_lctrl_pressed(true),
            PS2ScanCode::ReleasedLeftCtrl => {
                self.flags.set_lctrl_pressed(false)
            }
            PS2ScanCode::SuperKey => self.flags.set_superkey_pressed(true),
            PS2ScanCode::ReleasedSuperKey => {
                self.flags.set_superkey_pressed(false)
            }
            PS2ScanCode::CapsLock => {
                self.flags.set_capslock_pressed(
                    self.flags.is_capslock_pressed() ^ true,
                );
            }
            _ => {}
        }
    }

    pub fn read_raw_scancode(&self) -> Option<PS2ScanCode> {
        self.consumer.pop().map(PS2ScanCode::from)
    }

    /// TODO change in the future to just return the
    /// relevant ascii code and not a long str
    pub fn read_char(&self) -> Result<&'static str, PS2ScanCode> {
        let key = match self.read_raw_scancode() {
            Some(scancode) => scancode,
            None => return Err(PS2ScanCode::None),
        };
        let str = if self.flags.is_lshift_pressed()
            || self.flags.is_rshift_pressed()
            || self.flags.is_capslock_pressed()
        {
            key.to_str_shifted()
        } else {
            key.to_str()
        };
        if str.is_empty() { Err(key) } else { Ok(str) }
    }
}
