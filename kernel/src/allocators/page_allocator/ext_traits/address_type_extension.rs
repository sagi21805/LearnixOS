use super::paging_extension::PageEntryExtension;
use constants::enums::PageSize;
use cpu_utils::registers::get_current_page_table;
use cpu_utils::structures::paging::address_types::{PhysicalAddress, VirtualAddress};

pub(in super::super) trait VirtualAddressExtension {
    fn map(&self, address: PhysicalAddress, size: PageSize);
}

pub(in super::super) trait PhysicalAddressExtension {
    fn map(&self, address: VirtualAddress, size: PageSize);
}

impl PhysicalAddressExtension for PhysicalAddress {
    fn map(&self, address: VirtualAddress, page_size: PageSize) {
        address.map(self.clone(), page_size)
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
    fn map(&self, address: PhysicalAddress, page_size: PageSize) {
        if address.is_aligned(page_size.clone().alignment()) {
            let mut table = get_current_page_table();

            for table_number in ((page_size.clone() as usize + 1)..=4).rev() {
                let current_table_index = self.nth_pt_index(table_number);
                if table.entries[current_table_index].present() {
                    table = unsafe { table.entries[current_table_index].as_table_mut() }
                } else {
                    table = table.entries[self.nth_pt_index(table_number)].create_new_table();
                }
            }
            unsafe {
                table.entries[self.nth_pt_index(page_size as usize)].map_unchecked(address);
            }
        } else {
            panic!("address alignment doesn't match page type alignment, todo! raise a page fault")
        }
    }

}
