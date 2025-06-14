use core::{arch::x86_64::_MM_EXCEPT_INEXACT, cell::UnsafeCell, mem, ops::Add, slice};

use common::constants::{
    addresses::{
        MEMORY_MAP_LENGTH, MEMORY_MAP_OFFSET, PARSED_MEMORY_MAP_LENGTH, PARSED_MEMORY_MAP_OFFSET,
    },
    enums::MemoryRegionType,
};
use cpu_utils::structures::paging::address_types::PhysicalAddress;

use crate::println;

macro_rules! write_region {
    ($position:expr, $region:expr) => {
        (PARSED_MEMORY_MAP_OFFSET as *mut MemoryRegion)
            .add($position)
            .write_volatile($region);
        $position += 1;
    };
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct MemoryRegionExtended {
    pub base_address: u64,
    pub length: u64,
    pub region_type: MemoryRegionType,
    pub extended_attributes: u32,
}

#[derive(Debug)]
pub struct MemoryRegion {
    pub base_address: u64,
    pub length: u64,
    pub region_type: MemoryRegionType,
}

/// This function will parse the memory map provided by the bios
///
/// This memory map is provided in the constatnt address of the global [`MEMORY_MAP_OFFSET`]
///
/// The generated output will be saved to [`PARSED_MEMORY_MAP_OFFSET`],
/// and will include non gapped, organized entries of type [`MemoryRegion`]
pub fn parse_map() {
    let memory_map = unsafe {
        slice::from_raw_parts_mut(
            PhysicalAddress::new(MEMORY_MAP_OFFSET as usize)
                .translate()
                .as_mut_ptr::<MemoryRegionExtended>(),
            *(MEMORY_MAP_LENGTH as *const u32) as usize,
        )
    };
    let mut range_count = 0;
    let mut matched = unsafe { *memory_map.as_mut_ptr() };
    for region in memory_map {
        unsafe {
            match (region.region_type, matched.region_type) {
                (MemoryRegionType::Usable, MemoryRegionType::Usable) => {
                    if region.base_address > (matched.base_address + matched.length) {
                        write_region!(
                            range_count,
                            MemoryRegion {
                                ..(*(&matched as *const _ as *const MemoryRegion))
                            }
                        );
                        let inter_base = matched.base_address + matched.length;
                        write_region!(
                            range_count,
                            MemoryRegion {
                                base_address: inter_base,
                                length: region.base_address - inter_base,
                                region_type: MemoryRegionType::Reserved,
                            }
                        );
                    }
                    matched = *region;
                }
                (MemoryRegionType::Reserved, MemoryRegionType::Reserved) => {
                    matched.length = (region.base_address + region.length) - matched.base_address;
                }
                (MemoryRegionType::Usable, MemoryRegionType::Reserved)
                | (MemoryRegionType::Reserved, MemoryRegionType::Usable) => {
                    write_region!(
                        range_count,
                        MemoryRegion {
                            ..(*(&matched as *const _ as *const MemoryRegion))
                        }
                    );
                    matched = *region;
                }
                (_, _) => {
                    continue;
                }
            }
        }
    }
    if matched.region_type == MemoryRegionType::Usable {
        unsafe {
            write_region!(
                range_count,
                MemoryRegion {
                    ..(*(&matched as *const _ as *const MemoryRegion))
                }
            );
        }
    }
    unsafe { *(PARSED_MEMORY_MAP_LENGTH as *mut u32) = range_count as u32 }
}
