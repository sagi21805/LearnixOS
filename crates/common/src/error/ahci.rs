use crate::error::ConversionError;
use num_enum::TryFromPrimitive;
use strum_macros::EnumIter;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum HbaError {
    #[error("Address is not aligned properly")]
    AddressNotAligned,
}
