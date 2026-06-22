use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

pub struct LateInit<T>(MaybeUninit<T>);

impl<T> LateInit<T> {
    pub const fn uninit() -> LateInit<T> {
        LateInit::<T>(MaybeUninit::uninit())
    }

    pub const fn new(val: T) -> LateInit<T> {
        LateInit::<T>(MaybeUninit::new(val))
    }

    pub fn init(&self, val: T) -> &mut T {
        let ptr = self.0.as_ptr() as *mut T;
        unsafe {
            ptr.write_volatile(val);
            &mut *ptr
        }
    }

    pub const fn init_const(&self, val: T) -> &mut T {
        let ptr = self.0.as_ptr() as *mut T;
        unsafe {
            ptr.write(val);
            &mut *ptr
        }
    }

    pub const fn assume_init_ref(&self) -> &T {
        unsafe { self.0.assume_init_ref() }
    }

    pub const fn assume_init_mut(&mut self) -> &mut T {
        unsafe { self.0.assume_init_mut() }
    }
}

impl<T: Clone + Copy> LateInit<T> {
    pub const fn assume_init(&self) -> T {
        unsafe { self.0.assume_init() }
    }
}

#[rustfmt::skip]
impl<T> const Deref for LateInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.assume_init_ref() }
    }
}

#[rustfmt::skip]
impl<T> const DerefMut for LateInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.assume_init_mut() }
    }
}
