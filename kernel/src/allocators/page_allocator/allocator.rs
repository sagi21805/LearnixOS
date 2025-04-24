use super::address_type_extension::VirtualAddressExtension;
use super::{super::bitmap::BitMap, bitmap_extension::BitMapExtension};
use constants::{
    enums::PageSize,
    values::REGULAR_PAGE_SIZE,
};
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null,
};
use cpu_utils::registers::get_current_page_table;
use cpu_utils::structures::paging::{
    address_types::{PhysicalAddress, VirtualAddress},
};

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
    unsafe fn find_free_region(&self, size: PageSize) -> Option<(usize, u32)> {
        let bitmap = self.inner.as_ref_unchecked();

        match size {
            PageSize::Regular => {
                for index in bitmap.map.iter() {
                    let available_bit = index.trailing_ones();
                    if available_bit < u64::BITS {
                        return Some((index.clone() as usize, available_bit));
                    }
                }
                None
            }

            PageSize::Big | PageSize::Huge => {
                let index_alignment: usize = size.size_in_pages() / u64::BITS as usize;
                let map = &*(&*self.inner.as_ref_unchecked()).map;
                for (index, window) in map
                    .windows(index_alignment)
                    .step_by(index_alignment)
                    .enumerate()
                {
                    let sum: u64 = window.iter().sum();
                    if sum == 0 {
                        return Some((index, 0));
                    }
                }
                None
            }
        }
    }

    pub fn available_memory(&self) -> u64 {
        let bitmap = unsafe { self.inner.as_mut_unchecked() };
        return bitmap
            .map
            .iter()
            .map(|x| x.count_zeros() as u64)
            .sum::<u64>()
            * REGULAR_PAGE_SIZE as u64;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match PageSize::from_layout(layout) {
            Some(size) => match (
                self.find_free_region(size.clone()),
                get_current_page_table().find_available_page(size.clone()),
            ) {
                (Some((map_index, bit_index)), Some(virt)) => {
                    let bitmap = self.inner.as_mut_unchecked();
                    let phys = self.resolve_address(map_index, bit_index as usize);
                    bitmap.set_page_unchecked(map_index, bit_index, size.clone());
                    virt.map(phys.clone(), size);
                    unsafe {
                        return phys.as_ptr_mut();
                    }
                }

                (_, _) => null::<u8>() as *mut u8,
            },

            None => null::<u8>() as *mut u8,
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

unsafe impl Sync for PageAllocator {}
