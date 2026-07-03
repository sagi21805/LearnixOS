#![no_std]
#![feature(ptr_alignment_type)]

use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
};

use common::{
    address_types::{Address, PhysicalAddress},
    enums::MemoryRegionType,
};

use x86::memory_map::MemoryMap;

pub struct BumpAllocator<'a> {
    curser: AtomicUsize,
    mmap: &'a MemoryMap,
}

impl<'a> BumpAllocator<'a> {
    pub fn new(mmap: &'a MemoryMap) -> BumpAllocator<'a> {
        BumpAllocator {
            curser: AtomicUsize::new(0),
            mmap,
        }
    }
}

unsafe impl<'a> GlobalAlloc for BumpAllocator<'a> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut aligned_cursor = unsafe {
            PhysicalAddress::new_unchecked(
                self.curser.load(Ordering::Relaxed),
            )
            .align_up(layout.alignment())
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
                        .align_up(layout.alignment())
                    }
                }

                // Check that the allocation fits the block.
                aligned_cursor.as_usize() >= b.base_address as usize
                    && ((b.base_address + b.length) as usize)
                        .saturating_sub(layout.size())
                        >= aligned_cursor.as_usize()
            });

        if memmap_block.is_some() {
            self.curser.store(
                aligned_cursor.as_usize() + layout.size(),
                Ordering::Relaxed,
            );
            unsafe { aligned_cursor.as_non_null().as_mut() }
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!("Bump allocator does not support deallocation")
    }
}
