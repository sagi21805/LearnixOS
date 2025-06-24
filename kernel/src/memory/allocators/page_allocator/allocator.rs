use super::extension_traits::{BitMapExtension, PageSizeEnumExtension, VirtualAddressExtension};
use crate::memory::bitmap::{BitMap, Position};
use common::constants::values::REGULAR_PAGE_ALIGNMENT;
use common::constants::{enums::PageSize, values::REGULAR_PAGE_SIZE};
use core::mem::MaybeUninit;
use core::ptr::{self, null};
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
};
use cpu_utils::structures::paging::address_types::PhysicalAddress;
use cpu_utils::{
    registers::cr3::get_current_page_table, structures::paging::page_tables::PageTable,
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
        // allocator
    }

    unsafe fn map(&self) -> &BitMap {
        unsafe { self.0.as_ref_unchecked() }
    }

    unsafe fn map_mut(&self) -> &mut BitMap {
        unsafe { self.0.as_mut_unchecked() }
    }

    pub fn init(uninit: &mut MaybeUninit<Self>) {
        unsafe {
            // self.0.as_mut_unchecked().init();
            // self.0.as_mut_unchecked().set_bit_unchecked(0, 0); // set the first bit so zero is not counted;
            // Alloc reserved addresses TODO
            // get_current_page_table().map_physical_memory(
            // self.0.as_ref_unchecked().as_slice().len() * u64::BITS as usize * REGULAR_PAGE_SIZE,
            // );
        };
    }

    /// Resolves `map_index` and `bit_index` into actual physical address
    pub fn resolve_position(&self, p: &Position) -> PhysicalAddress {
        return PhysicalAddress::new(
            ((p.map_index * (u64::BITS as usize)) + p.bit_index as usize) * REGULAR_PAGE_SIZE,
        );
    }

    pub fn available_memory(&self) -> usize {
        unsafe { self.map().count_ones() * REGULAR_PAGE_SIZE }
    }

    /// Return the physical address of this table
    pub(super) fn alloc_table(&self) -> &'static mut PageTable {
        let free_block = unsafe { self.map().find_free_block(1) };

        match free_block {
            Some(p) => unsafe {
                let physical_address = self.resolve_position(&p);

                ptr::write(
                    physical_address.translate().as_mut_ptr::<PageTable>(),
                    PageTable::empty(),
                );

                self.map_mut().set_bit(&p);

                return &mut *physical_address.as_mut_ptr::<PageTable>();
            },

            None => panic!("No physical memory is available to allocate this table"),
        }
    }
}

pub trait LayoutExtension {
    fn into_page_count(&self) -> (PageSize, usize);
}

impl LayoutExtension for Layout {
    fn into_page_count(&self) -> (PageSize, usize) {
        
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match layout.align_to(REGULAR_PAGE_ALIGNMENT.as_usize()) {
            Ok(layout) => {
                let virt_region = PageTable::find_page_in_current_table(size.clone());
                let phys_region = self.find_free_region(size.clone());
                }
            }
            Err(_) => null::<u8>() as *mut u8,
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}

unsafe impl Sync for PageAllocator {}
