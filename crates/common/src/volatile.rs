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

macro_rules! impl_volatile_assign {
    ($trait:ident, $method:ident, $op_trait:ident, $op_method:ident) => {
        impl<T> core::ops::$trait<T> for Volatile<T>
        where
            T: core::ops::$op_trait<T, Output = T>,
        {
            fn $method(&mut self, rhs: T) {
                self.write(core::ops::$op_trait::$op_method(
                    self.read(),
                    rhs,
                ));
            }
        }
    };
}

impl_volatile_assign!(AddAssign, add_assign, Add, add);
impl_volatile_assign!(SubAssign, sub_assign, Sub, sub);
impl_volatile_assign!(MulAssign, mul_assign, Mul, mul);
impl_volatile_assign!(DivAssign, div_assign, Div, div);
impl_volatile_assign!(RemAssign, rem_assign, Rem, rem);

impl_volatile_assign!(BitAndAssign, bitand_assign, BitAnd, bitand);
impl_volatile_assign!(BitOrAssign, bitor_assign, BitOr, bitor);
impl_volatile_assign!(BitXorAssign, bitxor_assign, BitXor, bitxor);
impl_volatile_assign!(ShlAssign, shl_assign, Shl, shl);
impl_volatile_assign!(ShrAssign, shr_assign, Shr, shr);

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
