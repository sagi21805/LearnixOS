use crate::error::ConversionError;
use derive_more::Display;
use num_enum::{FromPrimitive, TryFromPrimitive, UnsafeFromPrimitive};

// ANCHOR: AHCIInterfaceSpeed
#[repr(u8)]
#[derive(
    PartialEq,
    Eq,
    Display,
    Clone,
    Copy,
    TryFromPrimitive,
    UnsafeFromPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum InterfaceSpeed {
    #[display("Device not present or communication not established")]
    DevNotPresent = 0,
    #[display("Gen1: 1.5Gb/s")]
    Gen1 = 1,
    #[display("Gen1: 3.0Gb/s")]
    Gen2 = 2,
    #[display("Gen1: 6.0Gb/s")]
    Gen3 = 3,
}
// ANCHOR_END: AHCIInterfaceSpeed

// ANCHOR: InterfaceCommunicationControl
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum InterfaceCommunicationControl {
    Idle = 0x0,
    Active = 0x1,
    Partial = 0x2,
    Slumber = 0x6,
    DevSleep = 0x8,
    #[num_enum(alternatives = [3..=5, 7, 9..=14])]
    Reserved = 0xf,
}
// ANCHOR_END: InterfaceCommunicationControl

// ANCHOR: DeviceType
#[repr(u32)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[num_enum(error_type(name = ConversionError<u32>, constructor = ConversionError::CantConvertFrom))]
pub enum DeviceType {
    SataDevice = 0x00000101,
    AtapiDevice = 0xeb140101,
    EnclosureManagementBridge = 0xc33c0101,
    PortMultiplier = 0x96690191,
}
// ANCHOR_END: DeviceType

// ANCHOR: InterfacePowerManagement
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum InterfacePowerManagement {
    DevNotPresent = 0,
    Active = 1,
    Partial = 2,
    Slumber = 6,
    DevSleep = 8,
}
// ANCHOR_END: InterfacePowerManagement

// ANCHOR: DeviceDetection
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum DeviceDetection {
    NotDetected = 0,
    DetectedNoCommunication = 1,
    Detected = 3,
    Device = 4,
}
// ANCHOR_END: device Detection

// ANCHOR: SpeedAllowed
#[repr(u8)]
#[derive(Display, Clone, Copy, TryFromPrimitive, UnsafeFromPrimitive)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum InterfaceSpeedRestriction {
    #[display("Device not present or communication not established")]
    NoRestriction = 0,
    #[display("Gen1: 1.5Gb/s")]
    Gen1 = 1,
    #[display("Gen1: 3.0Gb/s")]
    Gen2 = 2,
    #[display("Gen1: 6.0Gb/s")]
    Gen3 = 3,
}
// ANCHOR_END: SpeedAllowed

// ANCHOR: DeviceDetectionInitialization
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum InterfaceInitialization {
    NoInitializationRequested = 0,
    CommunicationInitialization = 1,
    DisableInterface = 4,
}
// ANCHOR_END: DeviceDetectionInitialization

// ANCHOR: FisTypes
#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum FisType {
    RegisterFisHost2Device = 0x27,
    RegisterFisDevice2Host = 0x34,
    DmaActivateFisDevice2Host = 0x39,
    DmaSetupFisBiDirectional = 0x41,
    DataFisBiDirectional = 0x46,
    BistActivateFisBiDirectional = 0x58,
    PioSetupFisDevice2Host = 0x5f,
    SetDevicesBits = 0xa1,
    _Reserved0 = 0xa6,
    _Reserved1 = 0xb8,
    _Reserved2 = 0xbf,
    _VendorSpecific0 = 0xc7,
    _VendorSpecific1 = 0xd4,
    _Reserved3 = 0xd9,

    #[num_enum(default)]
    Unknown = 0xff,
}
