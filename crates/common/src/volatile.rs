use core::fmt::Debug;

#[derive(Copy)]
#[repr(transparent)]
pub struct Volatile<T>(T);

impl<T> Volatile<T> {
    pub const fn new(vol: T) -> Volatile<T> {
        Volatile(vol)
    }

    /// Read from the hardware register
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    /// Write to the hardware register
    pub fn write(&mut self, value: T) {
        unsafe { core::ptr::write_volatile(&mut self.0 as *mut T, value) }
    }
}

impl<T> Clone for Volatile<T> {
    fn clone(&self) -> Self {
        Volatile(self.read())
    }
}

impl<T> Debug for Volatile<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.0 as *const T))
    }
}

#[macro_export]
macro_rules! read_volatile {
    ($arg: expr) => {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!($arg)) }
    };
}

#[macro_export]
macro_rules! write_volatile {
    ($arg: expr, $val: expr) => {
        unsafe {
            core::ptr::write_volatile(core::ptr::addr_of_mut!($arg), $val)
        }
    };
}
