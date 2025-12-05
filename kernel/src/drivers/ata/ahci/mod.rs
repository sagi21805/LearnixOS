use crate::drivers::pci::PciDevice;

#[derive(Copy, Clone)]
pub struct AHCIBaseAddress(u32);

impl AHCIBaseAddress {
    /// Bits 31-13 (Taken from AHCI Specification)
    pub fn base_address(&self) -> usize {
        const MASK: u32 = ((1 << 19) - 1) << 13;
        ((self.0 & MASK) >> 13) as usize
    }
}

pub fn initialize(device: &mut PciDevice) {}
