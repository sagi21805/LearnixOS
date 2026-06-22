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
    #[track_caller]
    fn relax(tick: usize) {
        // if tick > 10000 {
        core::hint::spin_loop();
        panic!("spin loop tick exceeded 10000");
        // }
    }
}
