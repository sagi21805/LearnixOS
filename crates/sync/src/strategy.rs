pub trait RelaxStrategy {
    /// This function will be called inside the locking loop until the lock
    /// is acquired.
    ///
    /// # Parameters
    ///
    /// * `tick` - The current tick count of the locking loop.
    fn relax(tick: usize);
}

pub struct Spin;

impl RelaxStrategy for Spin {
    fn relax(_tick: usize) {
        core::hint::spin_loop();
    }
}
