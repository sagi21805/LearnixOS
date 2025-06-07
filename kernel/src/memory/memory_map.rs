use common::constants::enums::MemoryRegionType;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct MemoryRegionExtended {
    pub base_address: u64,
    pub length: u64,
    pub region_type: MemoryRegionType,
    pub extended_attributes: u32,
}
