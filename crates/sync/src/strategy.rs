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
    /// Spins once. On bare metal this panics after 10000 ticks, because on
    /// one CPU the lock holder can't run again until this returns.
    #[track_caller]
    fn relax(tick: usize) {
        if cfg!(target_os = "none") && tick > 10000 {
            panic!("spin loop tick exceeded 10000");
        }
        core::hint::spin_loop();
    }
}
