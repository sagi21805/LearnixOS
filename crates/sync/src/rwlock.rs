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

    #[must_use]
    fn add_reader(&self) -> Option<()> {
        self.counter
            .try_update(Ordering::Acquire, Ordering::Relaxed, |v| {
                // No reader may enter while a writer holds the lock.
                if v & Self::WRITER != 0 {
                    return None;
                }
                v.checked_add(Self::READER)
            })
            .ok()
            .map(|_| ())
    }

    #[must_use]
    fn sub_reader(&self) -> Option<()> {
        self.counter
            .try_update(Ordering::Release, Ordering::Relaxed, |v| {
                v.checked_sub(Self::READER)
            })
            .ok()
            .map(|_| ())
    }

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

    #[must_use]
    fn sub_writer(&self) -> Option<()> {
        self.counter
            .compare_exchange(
                Self::WRITER,
                0,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_| ())
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
    pub fn read(&self) -> RwLockReadGuard<'_, T, R> {
        let mut tick = 0;
        loop {
            core::hint::spin_loop();
            match self.try_read() {
                Some(guard) => return guard,
                None => R::relax(tick),
            }
            tick += 1;
        }
    }

    /// Try to acquire a write lock without blocking.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T, R>> {
        self.lock
            .add_writer()
            .map(|_| RwLockWriteGuard { inner: self })
    }

    /// Acquire a write lock, spinning until it is available.
    pub fn write(&self) -> RwLockWriteGuard<'_, T, R> {
        let mut tick = 0;
        loop {
            match self.try_write() {
                Some(guard) => return guard,
                None => R::relax(tick),
            }
            tick += 1;
        }
    }
}

#[rustfmt::skip]
unsafe impl<T: Sized + Send + Sync, R: RelaxStrategy> Sync for RwLock<T, R> {}
unsafe impl<T: Sized + Send, R: RelaxStrategy> Send for RwLock<T, R> {}

pub struct RwLockReadGuard<'a, T, R: RelaxStrategy> {
    inner: &'a RwLock<T, R>,
}

unsafe impl<T: Sized, R: RelaxStrategy> Sync for RwLockReadGuard<'_, T, R> where
    for<'a> &'a T: Sync
{
}

unsafe impl<T: Sized, R: RelaxStrategy> Send for RwLockReadGuard<'_, T, R> where
    for<'a> &'a T: Send
{
}

impl<T, R: RelaxStrategy> Deref for RwLockReadGuard<'_, T, R> {
    type Target = T;

    fn deref(&self) -> &T { unsafe { &*self.inner.data.get() } }
}

impl<T, R: RelaxStrategy> Drop for RwLockReadGuard<'_, T, R> {
    fn drop(&mut self) {
        self.inner
            .lock
            .sub_reader()
            .expect("Couldn't remove reader");
    }
}

pub struct RwLockWriteGuard<'a, T, R: RelaxStrategy> {
    inner: &'a RwLock<T, R>,
}

unsafe impl<T: Sized, R: RelaxStrategy> Sync for RwLockWriteGuard<'_, T, R> where
    for<'a> &'a T: Sync
{
}

unsafe impl<T: Sized, R: RelaxStrategy> Send for RwLockWriteGuard<'_, T, R> where
    for<'a> &'a T: Send
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
    fn drop(&mut self) {
        self.inner
            .lock
            .sub_writer()
            .expect("Couldn't remove writer");
    }
}
