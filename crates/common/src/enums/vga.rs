use num_enum::{ConstIntoPrimitive, ConstTryFromPrimitive};

use crate::error::ConversionError;

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, ConstTryFromPrimitive, ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
/// All the colors coded per the VGA documentation
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(u8)]
pub enum VgaCommand {
    CursorOffsetHigh = 0xE,
    CursorOffsetLow = 0xF,
}
