use crate::error::ConversionError;
use num_enum::{ConstIntoPrimitive, ConstTryFromPrimitive};

// ANCHOR: sections
#[repr(u16)]
#[derive(
    Copy, Clone, Debug, ConstTryFromPrimitive, ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u16>, constructor = ConversionError::CantConvertFrom))]
pub enum Sections {
    Null = 0x0,
    KernelCode = 0x8,
    KernelData = 0x10,
    UserCode = 0x18,
    UserData = 0x20,
    TaskStateSegment = 0x28,
}
// ANCHOR_END: sections

// ANCHOR: segment_type
// Directly taken from Intel Software developer manual
// volume 3.
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, ConstTryFromPrimitive, ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum SystemSegmentType {
    TaskStateSegmentAvailable = 0b1001,
    CallGate = 0b1100,
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}
// ANCHOR_END: segment_type

// ANCHOR: segment_descriptor_type
#[repr(u8)]
#[derive(
    Copy, Clone, Debug, ConstTryFromPrimitive, ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum SegmentDescriptorType {
    System = 0,
    User = 1,
}
// ANCHOR_END: segment_descriptor_type
