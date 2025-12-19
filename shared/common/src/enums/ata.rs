use num_enum::TryFromPrimitive;

use crate::error::ConversionError;

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, TryFromPrimitive)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum AtaCommand {
    Nop = 0,
    ReadDmaExt = 0x25,
    IdentifyDevice = 0xec,
}
