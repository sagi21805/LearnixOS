//! A spin mutex that interrupt handlers can share with the code they
//! interrupt.

use core::ops::{Deref, DerefMut};

use sync::{
    mutex::{MutexGuard, SpinMutex},
    strategy::Spin,
};

use crate::instructions::interrupts::InterruptsDisabled;

/// A spin mutex that keeps interrupts off on this CPU while it is held,
/// so an interrupt handler that takes it can't deadlock against the code
/// it interrupted.
///
/// Guards must drop in the reverse order of locking. Dropping the outer
/// guard first turns interrupts back on while the inner lock is still
/// held.
///
/// ```no_run
/// use x86::interrupt_safe_mutex::InterruptSafeMutex;
///
/// static TICKS: InterruptSafeMutex<u64> = InterruptSafeMutex::new(0);
///
/// // In the timer handler, and anywhere else, alike.
/// *TICKS.lock() += 1;
/// ```
pub struct InterruptSafeMutex<T> {
    mutex: SpinMutex<T>,
}

impl<T> InterruptSafeMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            mutex: SpinMutex::new(data),
        }
    }

    /// Turns interrupts off, then takes the lock.
    ///
    /// Interrupts go off first, so no handler can run between taking the
    /// lock and holding it.
    #[track_caller]
    pub fn lock(&self) -> InterruptSafeMutexGuard<'_, T> {
        let interrupts = InterruptsDisabled::new();
        InterruptSafeMutexGuard {
            guard: self.mutex.lock(),
            _interrupts: interrupts,
        }
    }
}

/// Access to an [`InterruptSafeMutex`]'s data, with interrupts off until
/// it drops.
pub struct InterruptSafeMutexGuard<'a, T> {
    // Fields drop in declaration order, so the lock is released before
    // interrupts come back on.
    guard: MutexGuard<'a, T, Spin>,
    _interrupts: InterruptsDisabled,
}

impl<T> Deref for InterruptSafeMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T { &self.guard }
}

impl<T> DerefMut for InterruptSafeMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.guard }
}
