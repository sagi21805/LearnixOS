use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum HbaError {
    #[error("Address is not aligned properly")]
    AddressNotAligned,
}
