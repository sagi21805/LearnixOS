use crate::address_types::VirtualAddress;
use core::{num::NonZeroUsize, slice};

pub struct RingBuffer<T: 'static + Clone + Copy> {
    read_idx: usize,
    write_idx: usize,
    buffer: &'static mut [T],
}

impl<T: 'static + Clone + Copy> RingBuffer<T> {
    pub fn new(
        buffer_address: VirtualAddress,
        length: NonZeroUsize,
    ) -> Self {
        Self {
            read_idx: 0,
            write_idx: 0,
            buffer: unsafe {
                slice::from_raw_parts_mut(
                    buffer_address.as_non_null::<T>().as_mut(),
                    length.get(),
                )
            },
        }
    }

    pub fn write(&mut self, value: T) {
        self.buffer[self.write_idx] = value;
        self.write_idx = (self.write_idx + 1) % self.buffer.len();
    }

    pub fn read(&mut self) -> Option<T> {
        unsafe { core::arch::asm!("cli") };
        if self.write_idx != self.read_idx {
            unsafe { core::arch::asm!("sti") }
            let val = self.buffer[self.read_idx];
            self.read_idx = (self.read_idx + 1) % self.buffer.len();
            Some(val)
        } else {
            unsafe { core::arch::asm!("sti") }
            None
        }
    }
}
