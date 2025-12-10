use derive_more::Display;
use num_enum::{TryFromPrimitive, UnsafeFromPrimitive};

// ANCHOR: AHCIInterfaceSpeed
#[derive(Display, Clone, Copy, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum AHCIInterfaceSpeed {
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
