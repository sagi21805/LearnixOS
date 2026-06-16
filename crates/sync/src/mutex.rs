use core::{cell::UnsafeCell, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

pub struct SpinLockMutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLockMutex<T> where T: Send {}

impl<T> SpinLockMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> Guard<'_, T> {
        // While the lock is `true`, a swap to `true` returns the previous value which is `true`
        // which keeps the mutex locked.
        // Only when unlocking, the swap would return `false` which would stop the loop
        while self.locked.swap(true, Ordering::Acquire) {
            core::hint::spin_loop();
        }

        Guard { mutex: self }
    }

}

pub struct Guard<'a, T> {
    mutex: &'a SpinLockMutex<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}
