use thiserror::Error;

#[derive(Debug, Error)]
pub enum PciConfigurationError {
    #[error("Unknown Vendor: {}", _0)]
    UnknownVendor(u16),

    #[error("Unknown Device: {}", _0)]
    UnknownDevice(u16),

    #[error("Unknown ClassCode: {}", _0)]
    UnknownClassCode(u16),

    #[error("Unknown SubClass: {}", _0)]
    UnknownSubClass(u16),

    #[error("Unknown Programmable Interface: {}", _0)]
    UnknownProgrammableInterface(u16),

    #[error("Device on this bus is not existing. Bus: {} Device: {}", _0, _1)]
    NonExistentDevice(u8, u8),
}
