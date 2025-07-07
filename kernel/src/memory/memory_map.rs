use common::{
    constants::{KiB, MiB, PARSED_MEMORY_MAP_LENGTH},
    enums::MemoryRegionType,
};
use core::fmt::{self, Display, Formatter};

#[macro_export]
macro_rules! parsed_memory_map {
    () => {
        unsafe {
            use crate::memory::allocators::page_allocator::extensions::PhysicalAddressExt;
            ::core::slice::from_raw_parts_mut(
                common::address_types::PhysicalAddress::new(
                    common::constants::addresses::PARSED_MEMORY_MAP_OFFSET as usize,
                )
                .translate()
                .as_mut_ptr::<crate::memory::memory_map::MemoryRegion>(),
                *(common::address_types::PhysicalAddress::new(
                    common::constants::addresses::PARSED_MEMORY_MAP_LENGTH as usize,
                )
                .translate()
                .as_mut_ptr::<u32>()) as usize,
            )
        }
    };
}

#[macro_export]
macro_rules! raw_memory_map {
    () => {
        unsafe {
            use crate::memory::allocators::page_allocator::extensions::PhysicalAddressExt;
            ::core::slice::from_raw_parts_mut(
                common::address_types::PhysicalAddress::new(
                    common::constants::addresses::MEMORY_MAP_OFFSET as usize,
                )
                .translate()
                .as_mut_ptr::<crate::memory::memory_map::MemoryRegionExtended>(),
                *(common::address_types::PhysicalAddress::new(
                    common::constants::addresses::MEMORY_MAP_LENGTH as usize,
                )
                .translate()
                .as_mut_ptr::<u32>()) as usize,
            )
        }
    };
}

macro_rules! write_region {
    ($position:expr, $region:expr) => {
        (common::constants::PARSED_MEMORY_MAP_OFFSET as *mut MemoryRegion)
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

pub struct ParsedMapDisplay(pub &'static [MemoryRegion]);

impl Display for ParsedMapDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut usable = 0u64;
        let mut reserved = 0u64;

        for entry in self.0 {
            let size_mib = entry.length / MiB as u64;
            let size_kib = (entry.length - (size_mib * MiB as u64)) / KiB as u64;

            write!(
                f,
                "[0x{:0>9x} - 0x{:0>9x}]: type: {}",
                entry.base_address,
                entry.base_address + entry.length,
                entry.region_type as u32
            )?;

            match entry.region_type {
                MemoryRegionType::Usable => {
                    usable += entry.length;
                    writeln!(f, " (Size: {:>4} MiB{:>4} KiB)", size_mib, size_kib)?;
                }
                MemoryRegionType::Reserved => {
                    reserved += entry.length;
                    writeln!(f, " (Size: {:>4} MiB{:>4} KiB)", size_mib, size_kib)?;
                }
                _ => writeln!(f)?,
            }
        }

        let usable_mib = usable / MiB as u64;
        let usable_kib = (usable % MiB as u64) / KiB as u64;
        let reserved_mib = reserved / MiB as u64;
        let reserved_kib = (reserved % MiB as u64) / KiB as u64;

        writeln!(f)?;
        writeln!(
            f,
            "Total Usable Memory:   {:>5} MiB {:>4} KiB",
            usable_mib, usable_kib
        )?;
        writeln!(
            f,
            "Total Reserved Memory: {:>5} MiB {:>4} KiB",
            reserved_mib, reserved_kib
        )
    }
}

/// This function will parse the memory map provided by the bios
///
/// This memory map is provided in the constatnt address of the global [`MEMORY_MAP_OFFSET`]
///
/// The generated output will be saved to [`PARSED_MEMORY_MAP_OFFSET`],
/// and will include non gapped, organized entries of type [`MemoryRegion`]
pub fn parse_map() {
    let memory_map = raw_memory_map!();
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
