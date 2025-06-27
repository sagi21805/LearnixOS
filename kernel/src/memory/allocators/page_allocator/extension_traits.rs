use super::ALLOCATOR;
use common::constants::enums::PageSize;
use common::constants::values::{BIG_PAGE_SIZE, PAGE_DIRECTORY_ENTRIES};
use cpu_utils::structures::paging::address_types::{PhysicalAddress, VirtualAddress};
use cpu_utils::structures::paging::page_tables::{PageEntryFlags, PageTable, PageTableEntry};
use extend::ext;
#[ext]
impl PhysicalAddress {
    /// Maps this physical address to the specified virtual address with the given flags and page size.
    ///
    /// Delegates the mapping operation to the virtual address, associating it with this physical address.
    #[inline]
    fn map(&self, address: VirtualAddress, flags: PageEntryFlags, page_size: PageSize) {
        address.map(self.clone(), flags, page_size)
    }
}

#[ext]
pub impl PageTableEntry {
    /// Returns a mutable reference to the page table mapped by this entry, allocating and mapping a new table if necessary.
    ///
    /// If a page table is already mapped in this entry, it is returned. Otherwise, a new page table is allocated, mapped into this entry, and a mutable reference to it is returned. This guarantees that a valid page table is always provided.
    ///
    /// # Safety
    ///
    /// This function uses unsafe operations to allocate and map page tables. It assumes the presence of a global allocator and that the entry can be safely overwritten if not already mapped.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut entry = PageTableEntry::default();
    /// let table: &mut PageTable = entry.force_resolve_table_mut();
    /// ```
    fn force_resolve_table_mut(&mut self) -> &mut PageTable {
        if let Ok(table) = self.mapped_table_mut() {
            return table;
        } else {
            let resolved_table = unsafe { ALLOCATOR.assume_init_ref().alloc_table() };
            self.map_unchecked(
                PhysicalAddress(resolved_table.address().as_usize()),
                PageEntryFlags::table_flags(),
            );
            unsafe { &mut *self.mapped_unchecked().as_mut_ptr::<PageTable>() }
        }
    }
}

#[ext]
pub impl VirtualAddress {
    /// Maps this virtual address to the specified physical address in the current page table.
    ///
    /// If intermediate page tables required for this mapping do not exist, they are automatically created. Both the virtual and physical addresses must be aligned to the specified page size; otherwise, the function panics.
    ///
    /// # Parameters
    ///
    /// - `address`: The physical address to map to.
    /// - `flags`: Page entry flags to use for the mapping.
    /// - `page_size`: The page size for the mapping, determining the level in the page table hierarchy.
    ///
    /// # Panics
    ///
    /// Panics if either the virtual or physical address is not properly aligned for the given page size.
    ///
    /// # Examples
    ///
    /// ```
    /// let vaddr = VirtualAddress::new(0x4000_0000);
    /// let paddr = PhysicalAddress::new(0x2000_0000);
    /// vaddr.map(paddr, PageEntryFlags::PRESENT | PageEntryFlags::WRITABLE, PageSize::Regular);
    /// ```
    #[allow(static_mut_refs)]
    fn map(&self, address: PhysicalAddress, flags: PageEntryFlags, page_size: PageSize) {
        if address.is_aligned(page_size.alignment()) && self.is_aligned(page_size.alignment()) {
            let mut table = PageTable::current_table_mut(); // must use pointers becase can't reassign mut ref in a loop.
            for table_number in 0..(3 - page_size.clone() as usize) {
                let index = self.rev_nth_index_unchecked(table_number);
                let entry = &mut table.entries[index];
                let resolved_table = entry.force_resolve_table_mut();
                table = resolved_table;
            }
            table.entries[self.rev_nth_index_unchecked(3 - page_size as usize)]
                .map_unchecked(address, flags);
        } else {
            panic!("address alignment doesn't match page type alignment, todo! raise a page fault")
        }
    }
}

#[ext]
pub impl PageTable {
    /// Maps a contiguous region of physical memory starting at address 0 into the higher half of the virtual address space.
    ///
    /// The mapping begins at virtual address `0xffff800000000000` and covers the range from physical address 0 up to `mem_size_bytes`, using huge pages where possible. Intermediate page tables are created as needed. The number of top-level entries mapped is limited to 256.
    ///
    /// # Parameters
    /// - `mem_size_bytes`: The size in bytes of the physical memory region to map.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut page_table = PageTable::new();
    /// page_table.map_physical_memory(1024 * 1024 * 1024); // Maps 1 GiB of physical memory
    /// ```
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
        for forth_entry in &mut self.entries[(PAGE_DIRECTORY_ENTRIES / 2)
            ..(forth_level_entries_count + (PAGE_DIRECTORY_ENTRIES / 2))]
        {
            let third_table = forth_entry.force_resolve_table_mut();

            for third_entry in
                &mut third_table.entries[0..third_level_entries_count.min(PAGE_DIRECTORY_ENTRIES)]
            {
                let second_table = third_entry.force_resolve_table_mut();

                third_level_entries_count -= 1;
                for second_entry in &mut second_table.entries
                    [0..second_level_entries_count.min(PAGE_DIRECTORY_ENTRIES)]
                {
                    if !second_entry.present() {
                        second_entry.map(next_mapped.clone(), PageEntryFlags::huge_page_flags());
                    }
                    next_mapped += BIG_PAGE_SIZE.into();
                    second_level_entries_count -= 1;
                }
            }
        }
    }
}

#[ext]
pub impl PageSize {
    /// Returns the default page entry flags for the given page size.
    ///
    /// Regular pages use standard flags, while big and huge pages use huge page flags.
    ///
    /// # Examples
    ///
    /// ```
    /// let flags = PageSize::Regular.default_flags();
    /// assert_eq!(flags, PageEntryFlags::regular_page_flags());
    /// ```
    fn default_flags(&self) -> PageEntryFlags {
        match self {
            PageSize::Regular => PageEntryFlags::regular_page_flags(),
            PageSize::Big | PageSize::Huge => PageEntryFlags::huge_page_flags(),
        }
    }
    //     resolve_table!(&mut table.entries[index], PageEntryFlags::table_flags())
}
