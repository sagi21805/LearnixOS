use cpu_utils::structures::paging::page_tables::{PageTableEntry, PageTable};
use cpu_utils::structures::paging::address_types::{PhysicalAddress, VirtualAddress};

pub (in super) trait PagingAllocatorExtension {

    fn create_new_table(&self) -> &mut PageTable;

    fn get_next_table_mut(&self) -> &mut PageTable;

    fn map_table(&mut self, page_table: &'static PageTable);
}

impl PagingAllocatorExtension for PageTableEntry {

    #[inline]
    /// Return the physical address
    fn get_next_table_mut(&self) -> &mut PageTable {
        todo!();
        // if !self.huge_page() {
        //     unsafe {
        //         core::mem::transmute::<PhysicalAddress, &mut PageTable>(self.mapped_address())
        //     }
        // } else {
        //     panic!("The page is authoritative so there is no next table");
        // }
    }

    #[inline]
    fn map_table(&mut self, page_table: &'static PageTable) {
        // self.map(page_table.address().translate());
        todo!();
    }

    #[inline]
    /// create table, set it using set_next_table and return a reference to it
    fn create_new_table(&self) -> &mut PageTable {
        todo!();
        // let global_table = unsafe { get_current_page_table() };
        // let (p, v) = global_table.find_available_page(PageSize::Regular);
    }

}
