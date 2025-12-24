use crate::error::ConversionError;
use num_enum::TryFromPrimitive;
use strum_macros::EnumIter;
use thiserror::Error;

// ANCHOR: DiagnosticError

#[repr(u16)]
#[derive(Debug, Clone, Copy, TryFromPrimitive, Error, EnumIter)]
#[num_enum(error_type(name = ConversionError<u16>, constructor = ConversionError::CantConvertFrom))]
pub enum DiagnosticError {
    #[error("Physical ready signal changed state")]
    PhyRdyChange = 1 << 0,
    #[error("Internal error in the physical layer")]
    PhyInternal = 1 << 1,
    #[error("Communication wake signal detected")]
    CommWake = 1 << 2,
    #[error("10B to 8B decoding errors occurred")]
    DecodingError = 1 << 3,
    #[error("Disparity Error (Not use by AHCI)")]
    DisparityError = 1 << 4,
    #[error("One or more CRC errors occurred on link layer")]
    CrcError = 1 << 5,
    #[error("Handshake error, one or more R_ERR responses were received")]
    HandshakeError = 1 << 6,
    #[error("One or more link state machine errors were encountered")]
    LinkSequenceError = 1 << 7,
    #[error("Error on transport layer transition change")]
    TransportStateError = 1 << 8,
    #[error("One or more FISs were received with unknown type")]
    UnknownFisType = 1 << 9,
    #[error("A change in device presence has been detected")]
    Exchanged = 1 << 10,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, TryFromPrimitive, Error, EnumIter)]
#[num_enum(error_type(name = ConversionError<u16>, constructor = ConversionError::CantConvertFrom))]
pub enum AhciError {
    #[error("Data integrity error that occurred was recovered")]
    RecoveredDataIntegrityError = 1 << 0,
    #[error("Comm between device and host was lost and re-established")]
    RecoveredCommunicationError = 1 << 1,
    #[error("Data integrity error was occurred and NOT recovered")]
    DataIntegrityError = 1 << 8,
    #[error("A communication error that was not recovered occurred")]
    PersistentCommORDataIntegrityError = 1 << 9,
    #[error("A violation of the SATA protocol was detected")]
    ProtocolError = 1 << 10,
    #[error("The host bus adapter experienced an internal error")]
    InternalError = 1 << 11,
}

#[derive(Debug, Clone, Copy, Error)]
pub enum HbaError {
    #[error("Address is not aligned properly")]
    AddressNotAligned,
}
