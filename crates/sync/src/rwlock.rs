use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::strategy::{RelaxStrategy, Spin};

pub type SpinRwLock<T> = RwLock<T, Spin>;

#[repr(transparent)]
struct ReadWriteCounter {
    counter: AtomicUsize,
}

impl ReadWriteCounter {
    const WRITER: usize = 1 << 0;
    const READER: usize = 1 << 1;

    pub const fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }

    /// Enters as a reader, or `None` while a writer holds the lock.
    #[must_use]
    fn add_reader(&self) -> Option<()> {
        let previous =
            self.counter.fetch_add(Self::READER, Ordering::Acquire);
        if previous & Self::WRITER != 0 {
            // A writer holds the lock, so take the count back. `Relaxed`,
            // as this reader never touched the data.
            self.counter.fetch_sub(Self::READER, Ordering::Relaxed);
            return None;
        }
        Some(())
    }

    /// Leaves as a reader.
    fn sub_reader(&self) {
        let previous =
            self.counter.fetch_sub(Self::READER, Ordering::Release);
        debug_assert!(
            previous >= Self::READER,
            "a reader left that never entered"
        );
    }

    /// Enters as the writer, or `None` while anyone holds the lock.
    #[must_use]
    fn add_writer(&self) -> Option<()> {
        self.counter
            .compare_exchange(
                0,
                Self::WRITER,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_| ())
    }

    /// Whether a writer holds the lock. `Relaxed`, as it is only a hint
    /// for waiting.
    fn has_writer(&self) -> bool {
        self.counter.load(Ordering::Relaxed) & Self::WRITER != 0
    }

    /// Whether nobody holds the lock. `Relaxed`, as it is only a hint for
    /// waiting.
    fn is_free(&self) -> bool { self.counter.load(Ordering::Relaxed) == 0 }

    /// Leaves as the writer.
    ///
    /// This clears only the writer bit, because a reader backing out of
    /// `add_reader` may have its count in the counter right now.
    fn sub_writer(&self) {
        let previous =
            self.counter.fetch_and(!Self::WRITER, Ordering::Release);
        debug_assert!(
            previous & Self::WRITER != 0,
            "a writer left that never entered"
        );
    }
}

pub struct RwLock<T, R: RelaxStrategy> {
    lock: ReadWriteCounter,
    data: UnsafeCell<T>,
    strategy: PhantomData<R>,
}

impl<T, R: RelaxStrategy> RwLock<T, R> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: ReadWriteCounter::new(),
            data: UnsafeCell::new(data),
            strategy: PhantomData,
        }
    }

    /// Try to acquire a read lock without blocking.
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T, R>> {
        self.lock
            .add_reader()
            .map(|_| RwLockReadGuard { inner: self })
    }

    /// Acquire a read lock, spinning until it is available.
    ///
    /// While a writer holds the lock it waits on a plain load, and only
    /// tries again once the writer looks gone.
    #[track_caller]
    pub fn read(&self) -> RwLockReadGuard<'_, T, R> {
        let mut tick = 0;
        loop {
            if let Some(guard) = self.try_read() {
                return guard;
            }
            while self.lock.has_writer() {
                tick += 1;
                R::relax(tick);
            }
        }
    }

    /// Try to acquire a write lock without blocking.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T, R>> {
        self.lock
            .add_writer()
            .map(|_| RwLockWriteGuard { inner: self })
    }

    /// Acquire a write lock, spinning until it is available.
    ///
    /// While anyone holds the lock it waits on a plain load, and only
    /// tries again once the lock looks free.
    #[track_caller]
    pub fn write(&self) -> RwLockWriteGuard<'_, T, R> {
        let mut tick = 0;
        loop {
            if let Some(guard) = self.try_write() {
                return guard;
            }
            while !self.lock.is_free() {
                tick += 1;
                R::relax(tick);
            }
        }
    }
}

#[rustfmt::skip]
unsafe impl<T: Sized + Send + Sync, R: RelaxStrategy> Sync for RwLock<T, R> {}

pub struct RwLockReadGuard<'a, T, R: RelaxStrategy> {
    inner: &'a RwLock<T, R>,
}

// No `Send` impl on either guard. The derived one needs
// `RwLock<T, R>: Sync`, so `T: Send + Sync`.

// SAFETY: a shared read guard hands out only `&T`.
unsafe impl<T: Sync, R: RelaxStrategy> Sync for RwLockReadGuard<'_, T, R> {}

impl<T, R: RelaxStrategy> Deref for RwLockReadGuard<'_, T, R> {
    type Target = T;

    fn deref(&self) -> &T { unsafe { &*self.inner.data.get() } }
}

impl<T, R: RelaxStrategy> Drop for RwLockReadGuard<'_, T, R> {
    fn drop(&mut self) { self.inner.lock.sub_reader(); }
}

pub struct RwLockWriteGuard<'a, T, R: RelaxStrategy> {
    inner: &'a RwLock<T, R>,
}

// SAFETY: a shared write guard hands out only `&T`.
unsafe impl<T: Sync, R: RelaxStrategy> Sync
    for RwLockWriteGuard<'_, T, R>
{
}

impl<T, R: RelaxStrategy> Deref for RwLockWriteGuard<'_, T, R> {
    type Target = T;

    fn deref(&self) -> &T { unsafe { &*self.inner.data.get() } }
}

impl<T, R: RelaxStrategy> DerefMut for RwLockWriteGuard<'_, T, R> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.data.get() }
    }
}

impl<T, R: RelaxStrategy> Drop for RwLockWriteGuard<'_, T, R> {
    fn drop(&mut self) { self.inner.lock.sub_writer(); }
}
