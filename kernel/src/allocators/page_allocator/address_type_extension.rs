use constants::enums::PageSize;
use cpu_utils::structures::paging::address_types::{VirtualAddress, PhysicalAddress};
use super::paging_extension::PagingAllocatorExtension;
use super::super::get_current_page_table;

pub (in super) trait VirtualAddressExtension {
    fn map(&self, address: PhysicalAddress, size: PageSize);
}

pub (in super) trait PhysicalAddressExtension {
    fn map(&self, address: VirtualAddress, size: PageSize);
}

impl PhysicalAddressExtension for PhysicalAddress {
    fn map(&self, address: VirtualAddress, page_size: PageSize) {
        address.map(self.clone(), page_size)
    }
}

impl VirtualAddressExtension for VirtualAddress {

    /// Map this `virtual address` into the given `physical_address` with the given page_size
    ///
    /// # Parameters
    ///
    /// - `address`: The physical address to map this to, this address is needed
    /// - `page_size`: The size of the page from the [`PageSize`] enum
    ///
    /// # Safety
    ///
    /// This method assumes that the physical address given is not used in any other place
    /// and is owned only by this virtual address
    fn map(&self, address: PhysicalAddress, page_size: PageSize) {

        todo!();
        if address.is_aligned(page_size.clone().alignment()) {
            let mut table = get_current_page_table();
    
            for table_number in ((page_size.clone() as usize + 1)..=4).rev() {
                let current_table_index = self.nth_pt_index(table_number);
                if table.entries[current_table_index].present() {
                    table = table.entries[current_table_index].get_next_table_mut()
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
