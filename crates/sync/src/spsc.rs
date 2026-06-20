extern crate alloc;

use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use alloc::boxed::Box;

/// Single Producer Single Consumer Ring Buffer
pub struct SpscRingBuffer<T: Clone + Copy> {
    buffer: NonNull<[T]>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
    has_consumer: AtomicBool,
    has_producer: AtomicBool,
}

unsafe impl<T: Clone + Copy> Send for SpscRingBuffer<T> {}
unsafe impl<T: Clone + Copy> Sync for SpscRingBuffer<T> {}

impl<T: Clone + Copy> SpscRingBuffer<T> {
    pub fn new(buf: Box<[T]>) -> SpscRingBuffer<T> {
        SpscRingBuffer {
            capacity: buf.len(),
            buffer: Box::leak(buf).into(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            has_consumer: AtomicBool::new(false),
            has_producer: AtomicBool::new(false),
        }
    }

    pub fn try_utilize<'a>(
        &'a self,
    ) -> Option<(Producer<'a, T>, Consumer<'a, T>)> {
        if self.has_consumer.swap(true, Ordering::Acquire)
            && self.has_producer.swap(true, Ordering::Acquire)
        {
            return None;
        }
        Some((Producer { buf: self }, Consumer { buf: self }))
    }

    pub fn try_utilize_consumer<'a>(&'a self) -> Option<Consumer<'a, T>> {
        if self.has_consumer.swap(true, Ordering::Acquire) {
            return None;
        }
        Some(Consumer { buf: self })
    }

    pub fn try_utilize_producer<'a>(&'a self) -> Option<Producer<'a, T>> {
        if self.has_producer.swap(true, Ordering::Acquire) {
            return None;
        }
        Some(Producer { buf: self })
    }

    pub fn deutilize<'a>(
        &'a self,
        p: Producer<'a, T>,
        c: Consumer<'a, T>,
    ) {
        self.deutilize_consumer(c);
        self.deutilize_producer(p);
    }

    pub fn deutilize_consumer<'a>(&'a self, c: Consumer<'a, T>) {
        drop(c);
        self.has_consumer.store(false, Ordering::Release);
    }

    pub fn deutilize_producer<'a>(&'a self, p: Producer<'a, T>) {
        drop(p);
        self.has_producer.store(false, Ordering::Release);
    }
}

impl<T: Clone + Copy> Drop for SpscRingBuffer<T> {
    fn drop(&mut self) {
        todo!()
    }
}

/// Writes into the RingBuffer
pub struct Producer<'a, T: Clone + Copy> {
    buf: &'a SpscRingBuffer<T>,
}

impl<T: Clone + Copy> Producer<'_, T> {
    /// Pushes an item into the buffer, returning `None` if the buffer is
    /// full.
    pub fn push(&self, item: T) -> Option<()> {
        if self.buf.head.load(Ordering::Relaxed)
            - self.buf.tail.load(Ordering::Relaxed)
            < self.buf.capacity
        {
            return None;
        }

        let index =
            self.buf.head.load(Ordering::Relaxed) % self.buf.capacity;
        unsafe {
            // Elide mutablity
            let mut b = self.buf.buffer;
            b.as_mut()[index] = item;
        }
        self.buf.head.fetch_add(1, Ordering::Relaxed);
        Some(())
    }
}

impl<T: Clone + Copy> Drop for Producer<'_, T> {
    fn drop(&mut self) {
        self.buf.has_producer.store(false, Ordering::Release);
    }
}

/// Reads from the RingBuffer
pub struct Consumer<'a, T: Clone + Copy> {
    buf: &'a SpscRingBuffer<T>,
}

impl<T: Clone + Copy> Consumer<'_, T> {
    pub fn pop(&self) -> Option<T> {
        if self.buf.head.load(Ordering::Relaxed)
            - self.buf.tail.load(Ordering::Relaxed)
            == 0
        {
            return None;
        }

        let index =
            self.buf.tail.load(Ordering::Relaxed) % self.buf.capacity;
        unsafe {
            // Elide mutablity
            let item = self.buf.buffer.as_ref()[index];
            self.buf.tail.fetch_add(1, Ordering::Relaxed);
            Some(item)
        }
    }
}

impl<T: Clone + Copy> Drop for Consumer<'_, T> {
    fn drop(&mut self) {
        self.buf.has_consumer.store(false, Ordering::Release);
    }
}
