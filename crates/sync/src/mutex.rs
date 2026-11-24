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

    /// Takes the lock, spinning until it is free.
    ///
    /// While the lock is held it waits on a plain load, which shares the
    /// cache line with other waiters, and only swaps once the lock looks
    /// free.
    #[track_caller]
    pub fn lock(&self) -> MutexGuard<'_, T, R> {
        let mut tick = 0;
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            }
            // `Relaxed`, as this is only a hint. `try_lock` synchronizes.
            while self.locked.load(Ordering::Relaxed) {
                tick += 1;
                R::relax(tick);
            }
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T, R>> {
        if self.locked.swap(true, Ordering::Acquire) {
            return None;
        }
        Some(MutexGuard { mutex: self })
    }

    /// Releases the lock, whether or not a guard holds it.
    ///
    /// # Safety
    ///
    /// A guard that is still alive must not use the data again, because
    /// the next locker assumes it is alone.
    pub unsafe fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct MutexGuard<'a, T, R: RelaxStrategy> {
    mutex: &'a Mutex<T, R>,
}

// No `Send` impl. The derived one needs `Mutex<T, R>: Sync`, so `T: Send`.

// SAFETY: a shared guard hands out only `&T`.
unsafe impl<T: Sync, R: RelaxStrategy> Sync for MutexGuard<'_, T, R> {}

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
