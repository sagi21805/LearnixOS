use macros::bitfields;
use num_enum::{ConstIntoPrimitive, ConstTryFromPrimitive};

#[bitfields]
pub struct Nested {
    pub a: B1,
    #[flag(rw, flag_type = Test)]
    pub b: B3,
}

use core::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError<T: Debug + Sized> {
    #[error("Cannot convert from {:?}", _0)]
    CantConvertFrom(T),
}

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, ConstTryFromPrimitive, ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum Test {
    One = 1,
    SomeRandomName = 2,
}
