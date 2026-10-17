#![no_std]
#![feature(ptr_alignment_type)]
#![feature(const_default)]
#![feature(const_trait_impl)]

use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use common::{
    address_types::{Address, VirtualAddress},
    alloc::{Allocation, Allocations, BumpAllocations},
    constants::REGULAR_PAGE_ALIGNMENT,
    enums::MemoryRegionType,
};

use sync::mutex::SpinMutex;
use x86::memory_map::MemoryMap;

static ALLOCATIONS: SpinMutex<BumpAllocations> =
    SpinMutex::new(Allocations::default());

pub struct BumpAllocator<'a> {
    curser: SpinMutex<usize>,
    mmap: &'a MemoryMap,
    pub allocations: &'static SpinMutex<BumpAllocations>,
}

impl<'a> BumpAllocator<'a> {
    pub fn new(mmap: &'a MemoryMap) -> BumpAllocator<'a> {
        BumpAllocator {
            curser: SpinMutex::new(0),
            mmap,
            allocations: &ALLOCATIONS,
        }
    }
}

// SAFETY: blocks never overlap, because the cursor only moves forward,
// and nothing frees them.
unsafe impl<'a> Allocator for BumpAllocator<'a> {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let mut curser = self.curser.lock();

        // Allocate at least a page per allocation.
        let alignment = layout.alignment().max(REGULAR_PAGE_ALIGNMENT);

        let mut aligned_cursor = unsafe {
            VirtualAddress::new_unchecked(*curser).align_up(alignment)
        };

        let regions = self.mmap.regions.read();

        let memmap_block = regions
            .iter()
            .filter(|b| matches!(b.region_type, MemoryRegionType::Usable))
            .find(|b| {
                // If the cursor is before the block, advance it.
                if aligned_cursor.as_usize() < b.base_address as usize {
                    aligned_cursor = unsafe {
                        VirtualAddress::new_unchecked(
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
            self.allocations.lock().write(Allocation {
                layout,
                base: aligned_cursor,
            });
            Ok(NonNull::slice_from_raw_parts(
                aligned_cursor.as_non_null::<u8>(),
                layout.size(),
            ))
        } else {
            Err(AllocError)
        }
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        unimplemented!("Bump allocator does not support deallocation")
    }
}
