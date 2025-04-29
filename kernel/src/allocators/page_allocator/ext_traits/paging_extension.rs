use crate::allocators::page_allocator::ALLOCATOR;
use constants::enums::PageSize;
use core::alloc::{GlobalAlloc, Layout};
use core::{panic, ptr};
use cpu_utils::registers::get_current_page_table;
use cpu_utils::structures::paging::address_types::{PhysicalAddress, VirtualAddress};
use cpu_utils::structures::paging::page_tables::{PageTable, PageTableEntry};

pub(in super::super) trait PageEntryExtension {
    fn create_new_table(&self) -> &mut PageTable;
}

impl PageEntryExtension for PageTableEntry {
    #[inline]
    /// create table, set it using set_next_table and return a reference to it
    /// This method will allocate space on it's own
    fn create_new_table(&self) -> &mut PageTable {
        // unsafe {
        //     let table_address =
        //         ALLOCATOR.assume_init_mut().;
        //     // self.map_unchecked(PhysicalAddress(table_address as usize));

        //     ptr::write(table_address, PageTable::empty());
        // }
        todo!()
    }
}
