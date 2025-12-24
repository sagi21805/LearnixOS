use super::ALLOCATOR;
use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    constants::{
        BIG_PAGE_SIZE, PAGE_ALLOCATOR_OFFSET, PAGE_DIRECTORY_ENTRIES,
        PHYSICAL_MEMORY_OFFSET,
    },
    enums::{PageSize, PageTableLevel},
    error::EntryError,
};
use cpu_utils::structures::paging::{
    PageEntryFlags, PageTable, PageTableEntry,
};
use extend::ext;
use strum::VariantArray;

use common::error::TableError;
use cpu_utils::structures::paging::EntryIndex;

#[ext]
pub impl PhysicalAddress {
    fn map(
        &self,
        address: VirtualAddress,
        flags: PageEntryFlags,
        page_size: PageSize,
    ) {
        address.map(*self, flags, page_size)
    }

    fn translate(&self) -> VirtualAddress {
        unsafe {
            VirtualAddress::new_unchecked(
                PHYSICAL_MEMORY_OFFSET + self.as_usize(),
            )
        }
    }
}

#[ext]
pub impl PageTableEntry {
    /// This function will return a table mapped in this
    /// entry if there is one.
    ///
    /// Else, it will override what is inside the entry and
    /// map a new table to it so valid table is guaranteed
    /// to be returned.
    fn force_resolve_table_mut(&mut self) -> Option<&mut PageTable> {
        match self.mapped_table_mut() {
            Ok(table) => Some(table),
            Err(EntryError::NotATable) => None,
            Err(EntryError::NoMapping) => {
                let resolved_table =
                    unsafe { ALLOCATOR.assume_init_ref().alloc_table() };
                unsafe {
                    self.map_unchecked(
                        PhysicalAddress::new_unchecked(
                            resolved_table.address().as_usize(),
                        ),
                        PageEntryFlags::table_flags(),
                    );
                }
                unsafe {
                    Some(
                        &mut *self
                            .mapped_unchecked()
                            .as_mut_ptr::<PageTable>(),
                    )
                }
            }
        }
    }
}

#[ext]
pub impl VirtualAddress {
    /// Map this `virtual address` into the given
    /// `physical_address` with the current page table,
    /// obtained  from `cr3` if a page table for the
    /// given virtual address doesn't exist, a new table
    /// **will** be created for it
    ///
    /// # Parameters
    ///
    /// - `address`: The physical address to map this to, this address is
    ///   needed
    /// - `page_size`: The size of the page from the [`PageSize`] enum
    fn map(
        &self,
        address: PhysicalAddress,
        flags: PageEntryFlags,
        page_size: PageSize,
    ) {
        if address.is_aligned(page_size.alignment())
            && self.is_aligned(page_size.alignment())
        {
            let mut table = PageTable::current_table_mut();
            for level in
                PageTableLevel::VARIANTS[0..=page_size as usize].iter()
            {
                let index = self.index_of(*level);
                let entry = &mut table.entries[index];
                let resolved_table =
                    entry.force_resolve_table_mut().unwrap();
                table = resolved_table;
            }
            unsafe {
                table.entries[self.index_of(
                    PageTableLevel::VARIANTS[page_size as usize + 1],
                )]
                .map(address, flags);
            }
        } else {
            panic!(
                "address alignment doesn't match page type alignment, \
                 todo! raise a page fault"
            )
        }
    }

    fn set_flags(&self, flags: PageEntryFlags) -> Result<(), EntryError> {
        let page_size = PageSize::from_alignment(self.alignment())
            .expect("self address is not aligned to a page size");

        let mut table = PageTable::current_table_mut();

        for level in PageTableLevel::VARIANTS[0..page_size as usize].iter()
        {
            let index = self.index_of(*level);
            let entry = &mut table.entries[index];
            table = entry.mapped_table_mut()?;
        }
        table.entries[self
            .index_of(PageTableLevel::VARIANTS[page_size as usize + 1])]
        .set_flags(flags);
        Ok(())
    }

    fn translate(&self) -> PhysicalAddress {
        todo!()
    }
}

#[ext]
pub impl PageTable {
    /// Find an avavilable page in the given size.
    // ANCHOR: page_table_find_available_page
    #[cfg(target_arch = "x86_64")]
    fn find_available_page(
        page_size: PageSize,
    ) -> Result<VirtualAddress, TableError> {
        const TOTAL_LEVELS: usize = PageTableLevel::VARIANTS.len();
        let mut level_indices = [0usize; TOTAL_LEVELS];
        let mut page_tables = [Self::current_table(); TOTAL_LEVELS];
        let mut current_level = PageTableLevel::PML4;
        loop {
            let current_table =
                page_tables[TOTAL_LEVELS - current_level as usize];

            let ti = current_table.try_fetch_table(
                level_indices[TOTAL_LEVELS - current_level as usize],
                current_level,
                page_size,
            );

            let next_table = match ti {
                EntryIndex::OutOfEntries | EntryIndex::PageDoesNotFit => {
                    current_level = current_level.prev()?;
                    level_indices
                        [TOTAL_LEVELS - current_level as usize] += 1;
                    continue;
                }
                EntryIndex::Entry(entry) => {
                    level_indices[TOTAL_LEVELS - current_level as usize] =
                        entry.table_index();
                    unsafe {
                        &*entry.mapped_unchecked().as_ptr::<PageTable>()
                    }
                }
                EntryIndex::Index(i) => {
                    level_indices[TOTAL_LEVELS - current_level as usize] =
                        i;
                    return Ok(VirtualAddress::from_indices(
                        level_indices,
                    ));
                }
            };
            let next_level = current_level
                .next()
                .expect("Can't go next on a first level table");
            page_tables[TOTAL_LEVELS - next_level as usize] = next_table;
            current_level = next_level;
        }
    }
    // ANCHOR_END: page_table_find_available_page

    /// Map the region of memory from 0 to `mem_size_bytes`
    /// at the top of the page table so that ```rust
    /// VirtualAddress(0xffff800000000000) ->
    /// PhysicalAddress(0) ```
    ///
    /// TODO: ADD SUPPORT FOR FULL FLAG
    #[allow(unsafe_op_in_unsafe_fn)]
    fn map_physical_memory(&mut self, mem_size_bytes: usize) {
        let mut second_level_entries_count =
            (mem_size_bytes / BIG_PAGE_SIZE).max(1);
        let mut third_level_entries_count = second_level_entries_count
            .div_ceil(PAGE_ALLOCATOR_OFFSET)
            .max(1);
        let forth_level_entries_count = third_level_entries_count
            .div_ceil(PAGE_DIRECTORY_ENTRIES)
            .clamp(1, 256);
        let mut next_mapped = unsafe { PhysicalAddress::new_unchecked(0) };
        for forth_entry in &mut self.entries[(PAGE_DIRECTORY_ENTRIES / 2)
            ..(forth_level_entries_count + (PAGE_DIRECTORY_ENTRIES / 2))]
        {
            let third_table =
                forth_entry.force_resolve_table_mut().unwrap();

            for third_entry in &mut third_table.entries
                [0..third_level_entries_count.min(PAGE_DIRECTORY_ENTRIES)]
            {
                let second_table =
                    third_entry.force_resolve_table_mut().unwrap();

                third_level_entries_count -= 1;
                for second_entry in &mut second_table.entries[0
                    ..second_level_entries_count
                        .min(PAGE_DIRECTORY_ENTRIES)]
                {
                    if !second_entry.is_present() {
                        unsafe {
                            second_entry.map(
                                next_mapped,
                                PageEntryFlags::huge_page_flags(),
                            );
                        }
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
    fn default_flags(&self) -> PageEntryFlags {
        match self {
            PageSize::Regular => PageEntryFlags::regular_page_flags(),
            PageSize::Big | PageSize::Huge => {
                PageEntryFlags::huge_page_flags()
            }
        }
    }
}
