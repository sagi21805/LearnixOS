use super::super::bitmap::BitMap;
use constants::values::{
    BIG_PAGE_ALIGNMENT, BIG_PAGE_SIZE, HUGE_PAGE_ALIGNMENT, HUGE_PAGE_SIZE, REGULAR_PAGE_ALIGNMENT,
    REGULAR_PAGE_SIZE,
};
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null,
};
use cpu_utils::structures::paging::{address_types::{PhysicalAddress, VirtualAddress}, page_tables::PageTableEntry};
use super::paging_extension::PagingAllocatorExtension;

/// Sizes in the bitmap corresponding to how many bits are needed per page size
#[repr(u64)]
enum PageSizeAlignment {
    /// 4Kib Page
    Regular = 0,

    /// 2Mib Page
    Big = 8,

    /// 1Gib Page
    Huge = 8 * 512,
}

impl PageSizeAlignment {
    /// Determines the appropriate `PageSizeAlignment` for a given memory layout.
    ///
    /// # Parameters
    ///
    /// - `layout`: A [`Layout`] struct containing the memory size and alignment.
    pub const fn from_layout(layout: Layout) -> Option<PageSizeAlignment> {
        match (layout.size(), layout.align()) {
            (REGULAR_PAGE_SIZE, val) if val == REGULAR_PAGE_ALIGNMENT.as_usize() => {
                Some(PageSizeAlignment::Regular)
            }
            (BIG_PAGE_SIZE, val) if val == BIG_PAGE_ALIGNMENT.as_usize() => {
                Some(PageSizeAlignment::Big)
            }
            (HUGE_PAGE_SIZE, val) if val == HUGE_PAGE_ALIGNMENT.as_usize() => {
                Some(PageSizeAlignment::Huge)
            }

            _ => None,
        }
    }
}

#[derive(Debug)]
// TODO: This is not thread safe, probably should use Mutex in the future
/// Page allocator implemented with a bitmap, every bit corresponds to a physical page
/// This memory allocator assumes at least 256Kib of memory
pub struct PageAllocator {
    base_address: PhysicalAddress,
    inner: UnsafeCell<BitMap>,
}

impl PageAllocator {
    /// Creates a new allocator from the `bitmap_address` and the `memory_size`.
    ///
    /// # Parameters
    ///
    /// - `bitmap_address`: Virtual address that the bitmap will use to store the map
    /// - `base_memory_address`: The starting physical address of the contiguous memory block this allocator will manage
    /// - `memory_size`: The size of the memory block <u>in bytes</u>
    pub fn from_address_size(
        bitmap_address: VirtualAddress,
        base_memory_address: PhysicalAddress,
        memory_size: usize,
    ) -> PageAllocator {
        unsafe {
            let map_size = memory_size / (REGULAR_PAGE_SIZE * u64::BITS as usize);
            let bitmap = BitMap::new(bitmap_address, map_size);
            PageAllocator {
                base_address: base_memory_address,
                inner: UnsafeCell::new(bitmap),
            }
        }
    }

    /// Resolves `map_index` and `bit_index` into actual physical address
    pub fn resolve_address(&self, map_index: usize, bit_index: usize) -> PhysicalAddress {
        return self.base_address.clone()
            + ((map_index * (u64::BITS as usize)) + bit_index) * REGULAR_PAGE_SIZE;
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn find_free_region(&self, alignment: PageSizeAlignment) -> Option<(usize, u32)> {
        let bitmap = self.inner.as_ref_unchecked();

        match alignment {
            PageSizeAlignment::Regular => {
                for index in 0..bitmap.size {
                    let available_bit = bitmap.get_index_unchecked(index).trailing_ones();
                    if available_bit < 64 {
                        return Some((index, available_bit));
                    }
                }
                None
            }

            PageSizeAlignment::Big => {
                todo!()
            }

            PageSizeAlignment::Huge => {
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
        match PageSizeAlignment::from_layout(layout) {
            Some(alignment) => {
                match self.find_free_region(alignment) {
                    Some((map_index, bit_index)) => {
                        let bitmap = self.inner.as_mut_unchecked();
                        bitmap.set_bit_unchecked(map_index, bit_index);
                        unsafe {
                            return self.resolve_address(map_index, bit_index as usize).as_ptr_mut()
                        }
                    }

                    None => null::<u8>() as *mut u8,
                }
            },

            None => null::<u8>() as *mut u8,
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}

}

unsafe impl Sync for PageAllocator {}
