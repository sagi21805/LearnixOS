#![no_std]
#![feature(allocator_api)]
#![feature(ptr_alignment_type)]

use core::{
    alloc::{GlobalAlloc, Layout},
    cell::Cell,
};

use common::{
    address_types::{Address, PhysicalAddress},
    enums::MemoryRegionType,
};

use x86::memory_map::MemoryMap;

pub struct BumpAllocator<'a> {
    curser: Cell<PhysicalAddress>,
    mmap: &'a MemoryMap,
}

impl<'a> BumpAllocator<'a> {
    pub fn new(mmap: &'a MemoryMap) -> BumpAllocator<'a> {
        BumpAllocator {
            curser: Cell::new(unsafe {
                PhysicalAddress::new_unchecked(0)
            }),
            mmap,
        }
    }
}

unsafe impl<'a> GlobalAlloc for BumpAllocator<'a> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut aligned_cursor =
            self.curser.get().align_up(layout.alignment());

        let memmap_block = self
            .mmap
            .iter()
            .filter(|b| matches!(b.region_type, MemoryRegionType::Usable))
            .find(|b| {
                // If the cursor is before the block, advance it.
                if aligned_cursor.as_usize() < b.base_address as usize {
                    aligned_cursor = unsafe {
                        PhysicalAddress::new_unchecked(
                            b.base_address as usize,
                        )
                        .align_up(layout.alignment())
                    }
                }

                // Check that the allocation fits the block.
                aligned_cursor.as_usize() >= b.base_address as usize
                    && (b.base_address + b.length) as usize - layout.size()
                        >= aligned_cursor.as_usize()
            });

        if memmap_block.is_some() {
            self.curser.set(unsafe {
                PhysicalAddress::new_unchecked(
                    aligned_cursor.as_usize() + layout.size(),
                )
            });
            unsafe { aligned_cursor.as_non_null().as_mut() }
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unimplemented!("Bump allocator does not support deallocation")
    }
}
