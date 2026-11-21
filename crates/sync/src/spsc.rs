extern crate alloc;

use core::{
    alloc::Allocator,
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
    /// A ring buffer over `buf`, which it keeps for good, so `A` must
    /// live for good too.
    pub fn new<A: Allocator + 'static>(
        buf: Box<[T], A>,
    ) -> SpscRingBuffer<T> {
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

    /// The slot that position `index` lands on.
    fn slot(&self, index: usize) -> *mut T {
        // SAFETY: `index % capacity` is inside the buffer.
        unsafe {
            self.buffer.cast::<T>().as_ptr().add(index % self.capacity)
        }
    }
}

impl<T: Clone + Copy> Drop for SpscRingBuffer<T> {
    fn drop(&mut self) { todo!() }
}

/// Writes into the RingBuffer
pub struct Producer<'a, T: Clone + Copy> {
    buf: &'a SpscRingBuffer<T>,
}

impl<T: Clone + Copy> Producer<'_, T> {
    /// Pushes an item into the buffer, returning `None` if the buffer is
    /// full.
    pub fn push(&self, item: T) -> Option<()> {
        let head = self.buf.head.load(Ordering::Relaxed);
        // Pairs with the `Release` in `pop`, so the slot was read before
        // it is written again.
        let tail = self.buf.tail.load(Ordering::Acquire);
        if head - tail >= self.buf.capacity {
            return None;
        }

        // SAFETY: slots from head up to tail + capacity belong to the
        // producer.
        unsafe { self.buf.slot(head).write(item) };
        // Publishes the slot to the consumer's `Acquire` load.
        self.buf.head.store(head + 1, Ordering::Release);
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
    /// Takes the oldest item out of the buffer, returning `None` if the
    /// buffer is empty.
    pub fn pop(&self) -> Option<T> {
        let tail = self.buf.tail.load(Ordering::Relaxed);
        // Pairs with the `Release` in `push`, so the slot's write is
        // visible.
        let head = self.buf.head.load(Ordering::Acquire);
        if head == tail {
            return None;
        }

        // SAFETY: slots from tail up to head belong to the consumer.
        let item = unsafe { self.buf.slot(tail).read() };
        // Hands the slot back to the producer's `Acquire` load.
        self.buf.tail.store(tail + 1, Ordering::Release);
        Some(item)
    }
}

impl<T: Clone + Copy> Drop for Consumer<'_, T> {
    fn drop(&mut self) {
        self.buf.has_consumer.store(false, Ordering::Release);
    }
}
