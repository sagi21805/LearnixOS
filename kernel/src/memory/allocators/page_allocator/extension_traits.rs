use super::ALLOCATOR;
use crate::memory::bitmap::BitMap;
use common::constants::enums::PageSize;
use common::constants::values::{BIG_PAGE_SIZE, PAGE_DIRECTORY_ENTRIES};
use cpu_utils::registers::cr3::{cr3_read, get_current_page_table};
use cpu_utils::structures::paging::address_types::{
    PageTableWalk, PhysicalAddress, VirtualAddress,
};
use cpu_utils::structures::paging::page_tables::{PageEntryFlags, PageTable, PageTableEntry};

/// This macro will return a valid table from an entry.
/// If this entry has a mapped table, it will return it
/// else, it would allocate a new table, an map it to the entry.
#[macro_export]
macro_rules! resolve_table {
    ($entry:expr, $flags:expr) => {
        match $entry.as_table_mut() {
            None => {
                let resolved_table = ALLOCATOR.assume_init_ref().alloc_table();
                $entry.map_unchecked(PhysicalAddress(resolved_table.address().as_usize()), $flags);
                resolved_table
            }
            Some(table) => table,
        }
    };
}

pub(in super::super) trait BitMapExtension {
    unsafe fn set_page_unchecked(&mut self, map_index: usize, bit_index: u32, page_size: PageSize);
}

#[allow(unsafe_op_in_unsafe_fn)]
impl BitMapExtension for BitMap {
    unsafe fn set_page_unchecked(&mut self, map_index: usize, bit_index: u32, page_size: PageSize) {
        match page_size {
            PageSize::Regular => {
                self.set_bit_unchecked(map_index, bit_index);
            }

            PageSize::Big | PageSize::Huge => {
                for index in
                    map_index..(map_index + (page_size.size_in_pages() / u64::BITS as usize))
                {
                    self.set_index_unchecked(index);
                }
            }
        }
    }
}

pub(in super::super) trait VirtualAddressExtension {
    fn map(&self, address: PhysicalAddress, flags: PageEntryFlags, page_size: PageSize);
}

pub(in super::super) trait PhysicalAddressExtension {
    fn map(&self, address: VirtualAddress, flags: PageEntryFlags, page_size: PageSize);
}

impl PhysicalAddressExtension for PhysicalAddress {
    fn map(&self, address: VirtualAddress, flags: PageEntryFlags, page_size: PageSize) {
        address.map(self.clone(), flags, page_size)
    }
}

impl VirtualAddressExtension for VirtualAddress {
    /// Map this `virtual address` into the given `physical_address` with the current page table, obtained  from `cr3`
    /// if a page table for the given virtual address doesn't exist, a new table **will** be created for it
    ///
    /// # Parameters
    ///
    /// - `address`: The physical address to map this to, this address is needed
    /// - `page_size`: The size of the page from the [`PageSize`] enum
    #[allow(static_mut_refs)]
    fn map(&self, address: PhysicalAddress, flags: PageEntryFlags, page_size: PageSize) {
        if address.is_aligned(page_size.alignment()) && self.is_aligned(page_size.alignment()) {
            // println!("Called This Function");
            let mut table = get_current_page_table() as *mut PageTable; // must use pointers becase can't reassign mut ref in a loop.
            unsafe {
                for table_number in 0..(3 - page_size.clone() as usize) {
                    let index = self.rev_nth_index_unchecked(table_number);
                    let entry = &mut (*table).entries[index];
                    let resolved_table = resolve_table!(entry, PageEntryFlags::table_flags());
                    table = resolved_table as *mut PageTable;
                }
                (*table).entries[self.rev_nth_index_unchecked(3 - page_size as usize)]
                    .map_unchecked(address, flags);
            }
        } else {
            panic!("address alignment doesn't match page type alignment, todo! raise a page fault")
        }
    }
}

pub(in super::super) trait PageTableExtension {
    fn map_physical_memory(&mut self, mem_size_bytes: usize);
}

impl PageTableExtension for PageTable {
    /// Map the region of memory from 0 to `mem_size_bytes` at the top of the page table so that
    /// ```rust
    /// VirtualAddress(0xffff800000000000) -> PhysicalAddress(0)
    /// ```
    ///
    /// TODO: ADD SUPPORT FOR FULL FLAG
    #[allow(unsafe_op_in_unsafe_fn)]
    fn map_physical_memory(&mut self, mem_size_bytes: usize) {
        let mut second_level_entries_count = (mem_size_bytes / BIG_PAGE_SIZE).max(1);
        let mut third_level_entries_count =
            ((second_level_entries_count + PAGE_DIRECTORY_ENTRIES - 1) / PAGE_DIRECTORY_ENTRIES)
                .max(1);
        let forth_level_entries_count =
            (((third_level_entries_count + PAGE_DIRECTORY_ENTRIES - 1) / PAGE_DIRECTORY_ENTRIES)
                .max(1))
            .min(256);
        let mut next_mapped = PhysicalAddress(0);
        unsafe {
            for forth_entry in &mut self.entries[(PAGE_DIRECTORY_ENTRIES / 2)
                ..(forth_level_entries_count + (PAGE_DIRECTORY_ENTRIES / 2))]
            {
                let third_table = resolve_table!(forth_entry, PageEntryFlags::table_flags());

                for third_entry in &mut third_table.entries
                    [0..third_level_entries_count.min(PAGE_DIRECTORY_ENTRIES)]
                {
                    let second_table = resolve_table!(third_entry, PageEntryFlags::table_flags());

                    third_level_entries_count -= 1;
                    for second_entry in &mut second_table.entries
                        [0..second_level_entries_count.min(PAGE_DIRECTORY_ENTRIES)]
                    {
                        if !second_entry.present() {
                            second_entry.map_unchecked(
                                next_mapped.clone(),
                                PageEntryFlags::huge_page_flags(),
                            );
                        }
                        next_mapped += BIG_PAGE_SIZE;
                        second_level_entries_count -= 1;
                    }
                }
            }
        }
    }
}

pub(in super::super) trait PageSizeEnumExtension {
    fn default_flags(&self) -> PageEntryFlags;
}

impl PageSizeEnumExtension for PageSize {
    fn default_flags(&self) -> PageEntryFlags {
        match self {
            PageSize::Regular => PageEntryFlags::regular_page_flags(),
            PageSize::Big | PageSize::Huge => PageEntryFlags::huge_page_flags(),
        }
    }
    //     resolve_table!(&mut table.entries[index], PageEntryFlags::table_flags())
}
