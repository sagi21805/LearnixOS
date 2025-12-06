use derive_more::Display;

#[repr(u8)]
#[derive(Display, Debug, Clone, Copy)]
pub enum AHCIInterfaceSpeed {
    #[display("Gen1: 1.5Gb/s")]
    Gen1 = 1,
    #[display("Gen1: 3.0Gb/s")]
    Gen2 = 2,
    #[display("Gen1: 6.0Gb/s")]
    Gen3 = 3,
}
