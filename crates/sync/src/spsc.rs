extern crate alloc;

use core::{
    alloc::Allocator,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::boxed::Box;

/// Single Producer Single Consumer Ring Buffer
pub struct SpscRingBuffer<T: Copy> {
    buffer: NonNull<[T]>,
    /// The capacity minus one, so `index & mask` picks a slot.
    mask: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

// SAFETY: the buffer owns its slots, and `T: Send` lets them move.
unsafe impl<T: Copy + Send> Send for SpscRingBuffer<T> {}
// SAFETY: each item moves from the producer's thread to the consumer's,
// and only one thread touches a slot at a time.
unsafe impl<T: Copy + Send> Sync for SpscRingBuffer<T> {}

impl<T: Copy> SpscRingBuffer<T> {
    /// A ring buffer over `buf`, which it keeps for good, so `A` must
    /// live for good too.
    ///
    /// # Panics
    ///
    /// When the length of `buf` isn't a power of two.
    #[track_caller]
    pub fn new<A: Allocator + 'static>(
        buf: Box<[T], A>,
    ) -> SpscRingBuffer<T> {
        assert!(
            buf.len().is_power_of_two(),
            "a ring buffer's capacity is a power of two"
        );
        SpscRingBuffer {
            mask: buf.len() - 1,
            buffer: Box::leak(buf).into(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// The producer and consumer of this buffer.
    ///
    /// Both halves borrow the buffer, so it can't be split again while
    /// either is alive.
    ///
    /// ```compile_fail,E0499
    /// use sync::spsc::SpscRingBuffer;
    ///
    /// let mut buffer =
    ///     SpscRingBuffer::new(vec![0u8; 4].into_boxed_slice());
    /// let (producer, consumer) = buffer.split();
    /// // Doesn't compile, because the first pair still borrows `buffer`.
    /// let second = buffer.split();
    /// drop((producer, consumer));
    /// ```
    pub fn split(&mut self) -> (Producer<'_, T>, Consumer<'_, T>) {
        (Producer { buf: self }, Consumer { buf: self })
    }

    /// The slot that position `index` lands on.
    fn slot(&self, index: usize) -> *mut T {
        // SAFETY: `index & mask` is below the buffer's length.
        unsafe { self.buffer.cast::<T>().as_ptr().add(index & self.mask) }
    }
}

impl<T: Copy> Drop for SpscRingBuffer<T> {
    fn drop(&mut self) { todo!() }
}

/// Writes into the RingBuffer
pub struct Producer<'a, T: Copy> {
    buf: &'a SpscRingBuffer<T>,
}

impl<T: Copy> Producer<'_, T> {
    /// Pushes an item into the buffer, returning `None` if the buffer is
    /// full.
    pub fn push(&mut self, item: T) -> Option<()> {
        let head = self.buf.head.load(Ordering::Relaxed);
        // Pairs with the `Release` in `pop`, so the slot was read before
        // it is written again.
        let tail = self.buf.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) > self.buf.mask {
            return None;
        }

        // SAFETY: slots from head up to tail + capacity belong to the
        // producer.
        unsafe { self.buf.slot(head).write(item) };
        // Publishes the slot to the consumer's `Acquire` load.
        self.buf.head.store(head.wrapping_add(1), Ordering::Release);
        Some(())
    }
}

/// Reads from the RingBuffer
pub struct Consumer<'a, T: Copy> {
    buf: &'a SpscRingBuffer<T>,
}

impl<T: Copy> Consumer<'_, T> {
    /// Takes the oldest item out of the buffer, returning `None` if the
    /// buffer is empty.
    pub fn pop(&mut self) -> Option<T> {
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
        self.buf.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(item)
    }
}
