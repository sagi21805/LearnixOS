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
        let new = self.read_idx as isize + steps;

        if new > 0 {
            self.read_idx = new as usize % self.buffer.len();
        } else {
            self.read_idx = (self.buffer.len() as isize + new) as usize;
        }
    }

    pub unsafe fn advance_write(&mut self, steps: isize) {
        let new = self.write_idx as isize + steps;

        if new > 0 {
            self.write_idx = new as usize % self.buffer.len();
        } else {
            self.write_idx = (self.buffer.len() as isize + new) as usize;
        }
    }

    // Advances `idx` forward by `steps`, clamping so it never passes
    // `limit` for a ring buffer of length `len`.
    fn ring_advance_clamped(
        idx: usize,
        steps: usize,
        limit: usize,
        len: usize,
    ) -> usize {
        debug_assert!(len != 0);
        debug_assert!(idx < len, "idx {idx} must be < len {len}");

        if idx == limit {
            return idx;
        }
        let new = (idx + steps) % len;
        let overshot = if idx < limit {
            new >= limit || new < idx
        } else {
            new >= limit && new < idx
        };
        if overshot { limit } else { new }
    }

    // Returns the forward distance from `from` to `to` on a ring of `len`.
    fn ring_distance(from: usize, to: usize, len: usize) -> usize {
        if to >= from {
            to - from
        } else {
            len - from + to
        }
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

    // Return the amout of steps between the read and write pointer
    pub fn steps_between(&self) -> usize {
        Self::ring_distance(
            self.read_idx,
            self.write_idx,
            self.buffer.len(),
        )
    }

    // Reads `steps` elements from the ring buffer into `buffer`, returning
    // the number of elements read.
    pub fn read_bulk(&mut self, buffer: &mut [T]) -> usize {
        let steps = unsafe { self.read_bulk_no_advance(buffer) };
        self.forward_advance_read(steps);
        steps
    }

    /// Reads `steps` elements from the ring buffer into `buffer`,
    /// returning the number of elements read.
    ///
    /// # SAFETY
    ///
    /// This function does not increment the read pointer!
    pub unsafe fn read_bulk_no_advance(&self, buffer: &mut [T]) -> usize {
        let steps = buffer.len().min(self.steps_between());
        let len = self.buffer.len();
        let end = (self.read_idx + steps) % len;

        if end > self.read_idx || steps == 0 {
            buffer[..steps].copy_from_slice(
                &self.buffer[self.read_idx..self.read_idx + steps],
            );
        } else {
            let first_chunk = len - self.read_idx;
            buffer[..first_chunk]
                .copy_from_slice(&self.buffer[self.read_idx..]);
            buffer[first_chunk..steps]
                .copy_from_slice(&self.buffer[..end]);
        }
        steps
    }

    /// Advances the read index forward by `steps`, clamping at the write
    /// index.
    pub fn forward_advance_read(&mut self, steps: usize) {
        self.read_idx = Self::ring_advance_clamped(
            self.read_idx,
            steps,
            self.write_idx,
            self.buffer.len(),
        );
    }

    pub fn advance_to_write(&mut self) {
        self.read_idx = self.write_idx;
    }
}
