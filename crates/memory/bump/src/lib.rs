#![no_std]
#![feature(ptr_alignment_type)]

use core::alloc::{GlobalAlloc, Layout};

use common::{
    address_types::{Address, PhysicalAddress},
    constants::REGULAR_PAGE_ALIGNMENT,
    enums::MemoryRegionType,
};

use sync::mutex::SpinMutex;
use x86::memory_map::MemoryMap;

pub struct BumpAllocator<'a> {
    curser: SpinMutex<usize>,
    mmap: &'a MemoryMap,
}

impl<'a> BumpAllocator<'a> {
    pub fn new(mmap: &'a MemoryMap) -> BumpAllocator<'a> {
        BumpAllocator {
            curser: SpinMutex::new(0),
            mmap,
        }
    }
}

unsafe impl<'a> GlobalAlloc for BumpAllocator<'a> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut curser = self.curser.lock();

        // Allocate at least a page per allocation.
        let alignment = layout.alignment().max(REGULAR_PAGE_ALIGNMENT);

        let mut aligned_cursor = unsafe {
            PhysicalAddress::new_unchecked(*curser).align_up(alignment)
        };

        let regions = self.mmap.regions.read();

        let memmap_block = regions
            .iter()
            .filter(|b| matches!(b.region_type, MemoryRegionType::Usable))
            .find(|b| {
                // If the cursor is before the block, advance it.
                if aligned_cursor.as_usize() < b.base_address as usize {
                    aligned_cursor = unsafe {
                        PhysicalAddress::new_unchecked(
                            b.base_address as usize,
                        )
                        .align_up(alignment)
                    }
                }

                // Check that the allocation fits the block.
                aligned_cursor.as_usize() >= b.base_address as usize
                    && ((b.base_address + b.length) as usize)
                        .saturating_sub(layout.size())
                        >= aligned_cursor.as_usize()
            });

        if memmap_block.is_some() {
            *curser = aligned_cursor.as_usize() + layout.size();
            unsafe { aligned_cursor.as_non_null().as_mut() }
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!("Bump allocator does not support deallocation")
    }
}
