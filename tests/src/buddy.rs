#![allow(static_mut_refs)]

use buddy::{BuddyAllocator, meta::BuddyArena};
use common::enums::MemoryRegionType;
use page::Page;
use page::map::PageMap;
use sync::mutex::SpinMutex;
use x86::memory_map::{MemoryMap, MemoryRegion, MemoryRegionExtended};

pub static mut MOCK_UNPARSED_MEMORY_MAP: [MemoryRegionExtended; 7] = [
    MemoryRegionExtended {
        base_address: 0x0000_0000,
        length: 0x0009_FC00,
        region_type: MemoryRegionType::Usable,
        extended_attributes: 0,
    },
    MemoryRegionExtended {
        base_address: 0x0009_FC00,
        length: 0x0000_0400,
        region_type: MemoryRegionType::Reserved,
        extended_attributes: 0,
    },
    MemoryRegionExtended {
        base_address: 0x000F_0000,
        length: 0x0001_0000,
        region_type: MemoryRegionType::Reserved,
        extended_attributes: 0,
    },
    MemoryRegionExtended {
        base_address: 0x0010_0000,
        length: 0x07EE_0000,
        region_type: MemoryRegionType::Usable,
        extended_attributes: 0,
    },
    MemoryRegionExtended {
        base_address: 0x07FE_0000,
        length: 0x0000_3000,
        region_type: MemoryRegionType::Reclaimable,
        extended_attributes: 0,
    },
    MemoryRegionExtended {
        base_address: 0x07FE_4000,
        length: 0x0001_C000,
        region_type: MemoryRegionType::Reserved,
        extended_attributes: 0,
    },
    MemoryRegionExtended {
        base_address: 0x0800_0000,
        length: 0x0100_0000,
        region_type: MemoryRegionType::UserEnterd,
        extended_attributes: 0,
    },
];

static mut MOCK_MEMORY_MAP: [MemoryRegion; 32] =
    [MemoryRegion::default(); 32];

#[test]
fn test_buddy_allocator() {
    let mmap = unsafe {
        MemoryMap::parse_map(
            &mut MOCK_UNPARSED_MEMORY_MAP,
            &mut MOCK_MEMORY_MAP,
        )
        .unwrap()
    };

    println!("{}", mmap);
    let allocator = BuddyAllocator::<PageMap, Page>::new(&mmap);
}
