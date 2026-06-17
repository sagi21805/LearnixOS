use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::strategy::{RelaxStrategy, Spin};

pub struct SpinLockMutex<T, R: RelaxStrategy = Spin> {
    strategy: PhantomData<R>,
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T, S: RelaxStrategy> Sync for SpinLockMutex<T, S> where T: Send {}

impl<T> SpinLockMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            strategy: PhantomData,
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        // While the lock is `true`, a swap to `true` returns the previous
        // value which is `true` which keeps the mutex locked.
        // Only when unlocking, the swap would return `false` which would
        // stop the loop
        let mut tick: usize = 0;
        while self.locked.swap(true, Ordering::Acquire) {
            tick += 1;
            Spin::relax(tick);
        }

        SpinlockGuard { mutex: self }
    }
}

pub struct SpinlockGuard<'a, T> {
    mutex: &'a SpinLockMutex<T>,
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}
