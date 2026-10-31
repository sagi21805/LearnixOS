//! A value written once at runtime, then shared.

use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    pin::Pin,
    sync::atomic::{AtomicU8, Ordering},
};

const UNINIT: u8 = 0;
const INITIALIZING: u8 = 1;
const READY: u8 = 2;

/// A value written once, after which every reader shares it.
///
/// Its state goes `UNINIT`, `INITIALIZING`, `READY`. Only the first
/// initialization leaves `UNINIT`, so the value is written once and
/// nothing aliases the write. Readers see the value only once it is
/// `READY`, and the `Release` store of `READY` makes every write before
/// it visible to the `Acquire` load that sees `READY`.
///
/// ```
/// use sync::once::Once;
///
/// static ANSWER: Once<u32> = Once::uninit();
///
/// assert!(ANSWER.get().is_none());
/// ANSWER.init(42);
/// assert_eq!(*ANSWER, 42);
/// ```
pub struct Once<T> {
    state: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
}

// SAFETY: one thread writes the value before `READY`, and after it every
// thread only shares it, so `T` must be `Sync` to be shared and `Send`
// because it may be dropped by another thread than the one that made it.
unsafe impl<T: Send + Sync> Sync for Once<T> {}

impl<T> Once<T> {
    /// A `Once` with no value yet.
    pub const fn uninit() -> Once<T> {
        Once {
            state: AtomicU8::new(UNINIT),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Writes `value`, and returns it.
    ///
    /// # Panics
    ///
    /// When this `Once` was already initialized.
    #[track_caller]
    pub fn init(&self, value: T) -> &T { self.publish(value, |_| {}) }

    /// Writes `value`, runs `setup` on it in place, and only then makes
    /// it visible to readers.
    ///
    /// The value is pinned, because it never leaves this `Once`'s
    /// `'static` memory. A value that points into itself can be finished
    /// in `setup`.
    ///
    /// # Panics
    ///
    /// When this `Once` was already initialized.
    #[track_caller]
    pub fn init_with(
        &'static self,
        value: T,
        setup: impl FnOnce(Pin<&mut T>),
    ) -> &'static T {
        self.publish(value, |value| {
            // SAFETY: the value lives in `'static` memory that `Once`
            // never moves it out of.
            setup(unsafe { Pin::new_unchecked(value) })
        })
    }

    /// The value, or `None` before it is ready.
    pub fn get(&self) -> Option<&T> {
        (self.state.load(Ordering::Acquire) == READY)
            // SAFETY: `READY` means the value is written, and nothing
            // writes it again.
            .then(|| unsafe { (*self.value.get()).assume_init_ref() })
    }

    /// Claims the value, writes it, runs `setup` on it and marks it
    /// ready.
    #[track_caller]
    fn publish(&self, value: T, setup: impl FnOnce(&mut T)) -> &T {
        let claimed = self.state.compare_exchange(
            UNINIT,
            INITIALIZING,
            Ordering::Acquire,
            Ordering::Relaxed,
        );
        assert!(claimed.is_ok(), "a `Once` is initialized only once");

        // SAFETY: the claim makes this the only call that reaches the
        // value, and no reader sees it before `READY`.
        setup(unsafe { (*self.value.get()).write(value) });
        self.state.store(READY, Ordering::Release);

        // SAFETY: the value was written above, and from here on it is
        // only shared.
        unsafe { (*self.value.get()).assume_init_ref() }
    }
}

impl<T> Deref for Once<T> {
    type Target = T;

    #[track_caller]
    fn deref(&self) -> &T {
        self.get()
            .expect("a `Once` is read after it is initialized")
    }
}

impl<T> Drop for Once<T> {
    fn drop(&mut self) {
        if *self.state.get_mut() == READY {
            // SAFETY: `READY` means the value is written.
            unsafe { self.value.get_mut().assume_init_drop() };
        }
    }
}
