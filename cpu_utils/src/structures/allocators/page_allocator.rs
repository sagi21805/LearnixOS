use core::{
    alloc::{GlobalAlloc, Layout},
    borrow::BorrowMut,
    cell::UnsafeCell,
    panic,
    ptr::{Alignment, null},
};

// Requires minimal of 32kib of memory in jumps of 32kib
use constants::{
    enums::PageSize,
    values::{
        BIG_PAGE_ALIGNMENT, BIG_PAGE_SIZE, HUGE_PAGE_ALIGNMENT, HUGE_PAGE_SIZE,
        REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
};

use crate::structures::paging::address_types;
pub const TOTAL_MEMORY: usize = 0xffffffff;

#[derive(Debug)]
pub struct BitMap {
    map: *mut u64,
    size: usize,
}

#[allow(unsafe_op_in_unsafe_fn)]
impl BitMap {
    pub fn new(map_address: u64, memory_size: usize) -> BitMap {
        if memory_size.is_power_of_two() {
            unsafe {
                core::ptr::write_bytes(
                    map_address as *mut u64,
                    0,
                    (memory_size / REGULAR_PAGE_SIZE) / 64,
                )
            };
            BitMap {
                map: map_address as *mut u64,
                size: (memory_size / REGULAR_PAGE_SIZE) / 64,
            }
        } else {
            panic!("Memory is not a power of 2")
        }
    }

    fn div_mod(index: usize) -> (usize, u64) {
        // 3 = number of bit in (8 (because of u8) - 1) which is 0b111
        let map_index = index >> 3;
        let bit_index = (index & 0b111111) as u64;
        (map_index, bit_index)
    }

    /// The index represents the bit index in the whole array
    pub unsafe fn set_unchecked(&mut self, index: usize) {
        let (map_index, bit_index) = BitMap::div_mod(index);
        self.set_index_bit_unchecked(map_index, bit_index);
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> bool {
        let (division, bit) = BitMap::div_mod(index);
        (*self.map.add(division) & (1 << bit)) != 0
    }

    /// This will the the whole u64 number in that index to 1
    pub unsafe fn set_index_unchecked(&mut self, index: usize) {
        self.map.add(index).write_volatile(u64::MAX)
    }

    pub unsafe fn get_index_unchecked(&self, index: usize) -> u64 {
        self.map.add(index).read_volatile()
    }

    pub unsafe fn set_index_bit_unchecked(&mut self, map_index: usize, bit_index: u64) {
        *self.map.add(map_index) ^= 1 << bit_index;
    }

    pub unsafe fn get_bit_index_unchecked(&self, map_index: usize, bit_index: u64) -> bool {
        self.map.add(map_index).read_volatile() & (1 << bit_index) != 0
    }
}

#[repr(u64)]
enum PageSizeAlignment {
    Regular = 0,
    Big = 8,
    Huge = 8 * 512,
    None,
}

impl PageSizeAlignment {
    pub const fn from_layout(layout: Layout) -> PageSizeAlignment {
        match (layout.size(), layout.align()) {
            (REGULAR_PAGE_SIZE, val) if val == REGULAR_PAGE_ALIGNMENT.as_usize() => {
                PageSizeAlignment::Regular
            }
            (BIG_PAGE_SIZE, val) if val == BIG_PAGE_ALIGNMENT.as_usize() => PageSizeAlignment::Big,
            (HUGE_PAGE_SIZE, val) if val == HUGE_PAGE_ALIGNMENT.as_usize() => {
                PageSizeAlignment::Huge
            }

            _ => PageSizeAlignment::None,
        }
    }
}

#[derive(Debug)]
// TODO: This is not thread safe, probably should use Mutex in the future
pub struct PageAllocator {
    pub inner: UnsafeCell<BitMap>,
}

impl PageAllocator {
    pub fn from_address_size(bitmap_address: u64, memory_size: usize) -> PageAllocator {
        let bitmap = BitMap::new(bitmap_address, memory_size);
        PageAllocator {
            inner: UnsafeCell::new(bitmap),
        }
    }

    pub fn resolve_address(map_index: usize, bit_index: usize) -> *mut u8 {
        return (((map_index * 64) + bit_index) * REGULAR_PAGE_SIZE) as *mut u8;
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn find_free_region(&self, alignment: PageSizeAlignment) -> (usize, i32) {
        let bitmap = self.inner.as_ref_unchecked();

        match alignment {
            PageSizeAlignment::Regular => {
                for index in 0..bitmap.size {
                    let available_bit = bitmap.get_index_unchecked(index).trailing_ones() as i32;
                    if available_bit < 64 {
                        return (index, available_bit);
                    }
                }
                return (0, -1);
            }

            PageSizeAlignment::Big => {
                todo!()
            }

            PageSizeAlignment::Huge => {
                todo!()
            }

            PageSizeAlignment::None => {
                todo!()
            }
        }
    }

    pub fn available_memory(&self) -> u64 {
        let mut available_pages: u64 = 0;
        unsafe {
            let bitmap = self.inner.as_mut_unchecked();
            for i in 0..bitmap.size {
                available_pages += bitmap.get_index_unchecked(i).count_zeros() as u64;
            }
        }
        return available_pages * REGULAR_PAGE_SIZE as u64;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() < REGULAR_PAGE_SIZE || layout.align() < REGULAR_PAGE_ALIGNMENT.as_usize() {
            return null::<u8>() as *mut u8;
        } else {
            let (map_index, bit_index) =
                self.find_free_region(PageSizeAlignment::from_layout(layout));
            if bit_index >= 0 {
                let bitmap = self.inner.as_mut_unchecked();
                bitmap.set_index_bit_unchecked(map_index, bit_index as u64);
                PageAllocator::resolve_address(map_index, bit_index as usize)
            } else {
                return null::<u8>() as *mut u8;
            }
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

unsafe impl Sync for PageAllocator {}
