use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

pub struct LateInit<T>(MaybeUninit<T>);

impl<T> LateInit<T> {
    pub const fn uninit() -> LateInit<T> {
        LateInit::<T>(MaybeUninit::uninit())
    }

    pub const fn write(&mut self, val: T) {
        self.0.write(val);
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.assume_init_ref() }
    }
}

impl<T> DerefMut for LateInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.assume_init_mut() }
    }
}
