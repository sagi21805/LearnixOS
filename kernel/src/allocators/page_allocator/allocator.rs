use crate::allocators::bitmap;
use crate::println;

use super::extension_traits::{
    BitMapExtension, PageSizeEnumExtension, PhysicalAddressExtension, VirtualAddressExtension,
};
use crate::allocators::bitmap::BitMap;
use constants::{addresses::PHYSICAL_MEMORY_OFFSET, enums::PageSize, values::REGULAR_PAGE_SIZE};
use core::ptr::{self, null};
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
};
use cpu_utils::structures::paging::page_tables::PageEntryFlags;
use cpu_utils::{
    registers::cr3::cr3_write,
    structures::paging::address_types::{PhysicalAddress, VirtualAddress},
};
use cpu_utils::{
    registers::cr3::get_current_page_table,
    structures::paging::page_tables::{PageTable, PageTableEntry},
};

#[derive(Debug)]
// TODO: This is not thread safe, probably should use Mutex in the future
/// Page allocator implemented with a bitmap, every bit corresponds to a physical page
pub struct PageAllocator(UnsafeCell<BitMap>);

impl PageAllocator {
    /// Creates a new allocator from the `bitmap_address` and the `memory_size`.
    ///
    /// # Parameters
    ///
    /// - `bitmap_address`: Virtual address that is identity mapped and will use to store the map
    /// - `memory_size`: Memory size in <u>bytes</u>
    #[allow(unsafe_op_in_unsafe_fn)]
    pub const unsafe fn new(bitmap_address: PhysicalAddress, memory_size: usize) -> PageAllocator {
        let size_in_pages = memory_size / (REGULAR_PAGE_SIZE * u64::BITS as usize);
        PageAllocator(UnsafeCell::new(BitMap::new(bitmap_address, size_in_pages)))
        // let allocator = unsafe { PageAllocator(
        // ) };

        // allocator
    }

    pub fn init(&mut self) {
        unsafe {
            self.0.as_mut_unchecked().init();
            self.0.as_mut_unchecked().set_bit_unchecked(0, 0); // set the first bit so zero is not counted;
        };
    }

    /// Resolves `map_index` and `bit_index` into actual physical address
    pub fn resolve_address(&self, map_index: usize, bit_index: usize) -> PhysicalAddress {
        return PhysicalAddress::new(
            ((map_index * (u64::BITS as usize)) + bit_index) * REGULAR_PAGE_SIZE,
        );
    }

    fn find_free_big_or_huge_contiguous_region(
        &self,
        size: PageSize,
        n: usize,
    ) -> Option<(usize, u32)> {
        let bitmap = unsafe { self.0.as_ref_unchecked() };
        let index_alignment: usize = (size.size_in_pages() / u64::BITS as usize) * n.max(1);

        for (index, window) in bitmap
            .as_slice()
            .windows(index_alignment)
            .enumerate()
            .step_by(index_alignment)
        {
            let sum: u64 = window.iter().sum();
            if sum == 0 {
                return Some((index, 0));
            }
        }
        None
    }

    pub(self) fn find_free_contiguous_region(
        &self,
        size: PageSize,
        n: usize,
    ) -> Option<(usize, u32)> {
        let bitmap = unsafe { self.0.as_ref_unchecked() };
        match size {
            PageSize::Regular => {
                let mask = (1 << n) - 1;
                for (index, val) in bitmap.as_slice().iter().enumerate() {
                    for contiguous_bit in 0..=(u64::BITS - n as u32) {
                        if (val >> contiguous_bit) & mask == 0 {
                            return Some((index, contiguous_bit));
                        }
                    }
                }
                None
            }
            PageSize::Big | PageSize::Huge => self.find_free_big_or_huge_contiguous_region(size, n),
        }
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    pub(self) fn find_free_region(&self, size: PageSize) -> Option<(usize, u32)> {
        let bitmap = unsafe { self.0.as_ref_unchecked() };
        match size {
            PageSize::Regular => {
                for (index, val) in bitmap.as_slice().iter().enumerate() {
                    let available_bit = val.trailing_ones();
                    if available_bit < u64::BITS {
                        return Some((index, available_bit));
                    }
                }
                None
            }
            PageSize::Big | PageSize::Huge => self.find_free_big_or_huge_contiguous_region(size, 1),
        }
    }

    pub fn available_memory(&self) -> u64 {
        let bitmap = unsafe { self.0.as_mut_unchecked().as_slice() };
        return bitmap.iter().map(|x| x.count_zeros() as u64).sum::<u64>()
            * REGULAR_PAGE_SIZE as u64;
    }

    /// Return the physical address of this table
    pub(super) fn alloc_table(&self) -> &'static mut PageTable {
        let table_address = self.find_free_region(PageSize::Regular);

        match table_address {
            Some((map_index, bit_index)) => unsafe {
                let physical_address = self.resolve_address(map_index, bit_index as usize);

                ptr::write(
                    physical_address.translate().as_mut_ptr::<PageTable>(),
                    PageTable::empty(),
                );

                return &mut *physical_address.as_mut_ptr::<PageTable>();
            },

            None => panic!("No physical memory is available"),
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match PageSize::from_layout(layout) {
            Some(size) => {
                let virt_region = get_current_page_table().find_available_page(size.clone());
                let phys_region = self.find_free_region(size.clone());
                match (phys_region, virt_region) {
                    (Some((map_index, bit_index)), Some(virt)) => {
                        let bitmap = self.0.as_mut_unchecked();
                        let phys = self.resolve_address(map_index, bit_index as usize);
                        bitmap.set_page_unchecked(map_index, bit_index, size.clone());
                        virt.map(phys, size.default_flags(), size);
                        unsafe {
                            return virt.as_mut_ptr(); // SHOULD BE VIRT
                        }
                    }
                    (_, _) => null::<u8>() as *mut u8,
                }
            }

            None => null::<u8>() as *mut u8,
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

unsafe impl Sync for PageAllocator {}
