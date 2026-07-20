use core::{
    cell::UnsafeCell,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::strategy::{RelaxStrategy, Spin};

pub type SpinMutex<T> = Mutex<T, Spin>;

pub struct Mutex<T, R: RelaxStrategy> {
    strategy: PhantomData<R>,
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T, S: RelaxStrategy> Sync for Mutex<T, S> where T: Send {}

impl<T, R: RelaxStrategy> Mutex<T, R> {
    pub const fn new(data: T) -> Self {
        Self {
            strategy: PhantomData,
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    // TODO: Make safe with compile time state of locked.
    pub const unsafe fn new_locked(data: T) -> Self {
        Self {
            strategy: PhantomData,
            locked: AtomicBool::new(true),
            data: UnsafeCell::new(data),
        }
    }

    #[track_caller]
    pub fn lock(&self) -> MutexGuard<'_, T, R> {
        // While the lock is `true`, a swap to `true` returns the previous
        // value which is `true` which keeps the mutex locked.
        // Only when unlocking, the swap would return `false` which would
        // stop the loop
        let mut tick: usize = 0;
        while self.locked.swap(true, Ordering::Acquire) {
            tick += 1;
            Spin::relax(tick);
        }

        MutexGuard { mutex: self }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T, R>> {
        if self.locked.swap(true, Ordering::Acquire) {
            return None;
        }
        Some(MutexGuard { mutex: self })
    }

    // TODO: Make safe with so can only be called when in locked state.
    pub const unsafe fn leak(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    // Release the lock
    pub unsafe fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct MutexGuard<'a, T, R: RelaxStrategy> {
    mutex: &'a Mutex<T, R>,
}

unsafe impl<T: Sized, R: RelaxStrategy> Sync for MutexGuard<'_, T, R> where
    for<'a> &'a T: Sync
{
}
unsafe impl<T: Sized, R: RelaxStrategy> Send for MutexGuard<'_, T, R> where
    for<'a> &'a T: Send
{
}

impl<T, R: RelaxStrategy> Deref for MutexGuard<'_, T, R> {
    type Target = T;

    fn deref(&self) -> &Self::Target { unsafe { &*self.mutex.data.get() } }
}

impl<T, R: RelaxStrategy> DerefMut for MutexGuard<'_, T, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T, R: RelaxStrategy> Drop for MutexGuard<'_, T, R> {
    fn drop(&mut self) {
        unsafe {
            self.mutex.force_unlock();
        }
    }
}

impl<'a, T, R> Debug for MutexGuard<'a, T, R>
where
    T: Sized + Debug,
    R: RelaxStrategy,
{
    fn fmt(
        &self,
        f: &mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result {
        f.debug_struct("MutexGuard")
            .field("data", &&**self)
            .finish()
    }
}
