use core::task::Waker;

use x86::interrupt_safe_mutex::InterruptSafeMutex;

/// The waker of the one task that waits on an interrupt.
///
/// The task stores its waker and the interrupt handler takes it, so the
/// lock keeps interrupts off while it is held.
pub struct WakerSlot(InterruptSafeMutex<Option<Waker>>);

impl WakerSlot {
    pub const fn new() -> Self { Self(InterruptSafeMutex::new(None)) }

    /// Stores `waker` for the next interrupt, unless it is the one stored
    /// already.
    pub fn register(&self, waker: &Waker) {
        let mut slot = self.0.lock();
        if !slot.as_ref().is_some_and(|stored| stored.will_wake(waker)) {
            *slot = Some(waker.clone());
        }
    }

    /// Wakes the stored waker, if any, after releasing the lock, so its
    /// `wake` never runs under this lock.
    pub fn wake(&self) {
        let waker = self.0.lock().take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl Default for WakerSlot {
    fn default() -> Self { Self::new() }
}
