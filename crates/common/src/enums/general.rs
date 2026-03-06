use crate::error::ConversionError;
use num_enum::{ConstIntoPrimitive, ConstTryFromPrimitive};

// ANCHOR: dpl
#[repr(u8)]
#[derive(
    Debug, Clone, Copy, ConstTryFromPrimitive, ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum ProtectionLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}
// ANCHOR_END: dpl
