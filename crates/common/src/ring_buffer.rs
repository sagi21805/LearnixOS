extern crate alloc;

use alloc::boxed::Box;

pub struct RingBuffer<T: 'static + Clone + Copy> {
    read_idx: usize,
    write_idx: usize,
    buffer: Box<[T]>,
}

impl<T: 'static + Clone + Copy> RingBuffer<T> {
    pub fn new(buffer: Box<[T]>) -> Self {
        Self {
            read_idx: 0,
            write_idx: 0,
            buffer,
        }
    }

    pub fn read_idx(&self) -> usize {
        self.read_idx
    }

    pub fn write_idx(&self) -> usize {
        self.write_idx
    }

    pub unsafe fn advance_read(&mut self, steps: isize) {
        self.read_idx =
            (self.read_idx as isize + steps) as usize % self.buffer.len();
    }

    pub unsafe fn advance_write(&mut self, steps: isize) {
        self.write_idx =
            (self.write_idx as isize + steps) as usize % self.buffer.len();
    }

    pub fn write(&mut self, value: T) {
        self.buffer.as_mut()[self.write_idx] = value;
        self.write_idx = (self.write_idx + 1) % self.buffer.len();
    }

    // TODO: remove sti and cli from here to the keyboard or interrupt
    // handle logic
    pub fn read(&mut self) -> Option<T> {
        if self.write_idx == self.read_idx {
            return None;
        }

        let val = self.buffer.as_mut()[self.read_idx];
        self.read_idx = (self.read_idx + 1) % self.buffer.len();

        Some(val)
    }

    // Advances the read index forward by `steps` steps.
    //
    // This operation ensures the read idx will not pass the write index.
    pub fn forward_advance_read(&mut self, steps: usize) {
        if self.read_idx == self.write_idx {
            return;
        }

        debug_assert!(self.buffer.len() != 0);

        let new = (self.read_idx + steps) % self.buffer.len();

        let overshot = if self.read_idx < self.write_idx {
            new >= self.write_idx || new < self.read_idx
        } else {
            new >= self.write_idx && new < self.read_idx
        };

        self.read_idx = if overshot { self.write_idx } else { new };
    }

    pub fn advance_to_write(&mut self) {
        self.read_idx = self.write_idx;
    }
}
