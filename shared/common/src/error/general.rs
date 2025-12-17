use core::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError<T: Debug + Sized> {
    #[error("Cannot convert from {:?}", _0)]
    CantConvertFrom(T),
}
