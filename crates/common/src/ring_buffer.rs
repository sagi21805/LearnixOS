use core::ptr::NonNull;

pub struct RingBuffer<T: 'static + Clone + Copy> {
    read_idx: usize,
    write_idx: usize,
    buffer: NonNull<[T]>,
}

impl<T: 'static + Clone + Copy> RingBuffer<T> {
    pub fn new(buffer: NonNull<[T]>) -> Self {
        Self {
            read_idx: 0,
            write_idx: 0,
            buffer,
        }
    }

    pub fn write(&mut self, value: T) {
        unsafe { self.buffer.as_mut()[self.write_idx] = value };
        self.write_idx = (self.write_idx + 1) % self.buffer.len();
    }

    // TODO: remove sti and cli from here to the keyboard or interrupt
    // handle logic
    pub fn read(&mut self) -> Option<T> {
        if self.write_idx == self.read_idx {
            return None;
        }

        let val = unsafe { self.buffer.as_mut()[self.read_idx] };
        self.read_idx = (self.read_idx + 1) % self.buffer.len();

        Some(val)
    }
}
