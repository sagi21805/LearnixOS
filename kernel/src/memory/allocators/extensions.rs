use core::{num::NonZero, ptr::NonNull};

use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    constants::{
        BIG_PAGE_SIZE, HUGE_PAGE_SIZE, PAGE_DIRECTORY_ENTRIES,
        PHYSICAL_MEMORY_OFFSET,
    },
    enums::{PageSize, PageTableLevel},
    error::EntryError,
    late_init::LateInit,
};
use cpu_utils::structures::paging::{
    PageEntryFlags, PageTable, PageTableEntry,
};
use extend::ext;
use strum::VariantArray;

use common::error::TableError;
use cpu_utils::structures::paging::EntryIndex;

use crate::memory::{
    allocators::buddy::BUDDY_ALLOCATOR, page::map::PageMap,
};

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
    fn force_resolve_table_mut(&mut self) -> Option<NonNull<PageTable>> {
        match self.mapped_table() {
            Ok(table) => Some(table),
            Err(EntryError::NotATable) => None,
            Err(EntryError::NoMapping) => unsafe {
                let resolved_table = BUDDY_ALLOCATOR.alloc_table();
                self.map_unchecked(
                    PhysicalAddress::new_unchecked(
                        resolved_table.addr().get(),
                    ),
                    PageEntryFlags::table_flags(),
                );
                Some(self.mapped_unchecked().as_non_null::<PageTable>())
            },
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
            let mut table = PageTable::current_table();
            for level in
                PageTableLevel::VARIANTS[0..=page_size as usize].iter()
            {
                let index = self.index_of(*level);
                let entry = unsafe { &mut table.as_mut().entries[index] };
                let resolved_table = entry
                    .force_resolve_table_mut()
                    .expect("Tried to create table on a mapped entry");
                table = resolved_table;
            }
            unsafe {
                table.as_mut().entries[self.index_of(
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

    fn set_flags(
        &self,
        flags: PageEntryFlags,
        page_size: PageSize,
        num_pages: NonZero<usize>,
    ) -> Result<(), EntryError> {
        let address_index = self.index_of(page_size.min_level());

        debug_assert!(
            address_index + num_pages.get() <= PAGE_DIRECTORY_ENTRIES,
            "There are only 512 entries inside a table"
        );

        let mut table = self.walk(page_size.min_level())?;

        unsafe {
            table
                .as_mut()
                .entries
                .iter_mut()
                .skip(address_index)
                .take(num_pages.get())
                .for_each(|entry| entry.set_flags(flags));
        }

        Ok(())
    }

    /// Return the entry that is pointed by the wanted level
    fn walk(
        &self,
        wanted: PageTableLevel,
    ) -> Result<NonNull<PageTable>, EntryError> {
        let mut table = PageTable::current_table();

        for level in PageTableLevel::VARIANTS[0..wanted as usize].iter() {
            let entry =
                unsafe { &table.as_ref().entries[self.index_of(*level)] };
            table = entry.mapped_table()?;
        }

        Ok(table)
    }

    fn translate(&self) -> Option<PhysicalAddress> {
        let mut table = PageTable::current_table();

        for level in PageTableLevel::VARIANTS.iter() {
            let entry =
                unsafe { &table.as_mut().entries[self.index_of(*level)] };
            match entry.mapped_table() {
                Ok(mapped) => table = mapped,
                Err(EntryError::NotATable) => {
                    return unsafe { Some(entry.mapped_unchecked()) };
                }
                Err(EntryError::NoMapping) => return None,
            }
        }
        unreachable!()
    }
}

#[ext]
pub impl PageTable {
    // TODO: trn into a tail called function with become
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
            let mut current_table =
                page_tables[TOTAL_LEVELS - current_level as usize];

            let ti = unsafe {
                current_table.as_mut().try_fetch_table(
                    level_indices[TOTAL_LEVELS - current_level as usize],
                    current_level,
                    page_size,
                )
            };

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
                        entry.mapped_unchecked().as_non_null::<PageTable>()
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

    // TODO: turn into a tail called function with become
    /// Map the region of memory from 0 to `mem_size_bytes`
    /// at the top of the page table so that
    ///
    /// ```rust
    /// VirtualAddress(0xffff800000000000) -> PhysicalAddress(0)
    /// ```
    ///
    /// TODO: ADD SUPPORT FOR FULL FLAG
    #[allow(unsafe_op_in_unsafe_fn)]
    fn map_physical_memory(&mut self, mem_size_bytes: usize) {
        let mut second_level_entries_count =
            (mem_size_bytes / BIG_PAGE_SIZE) + 1;
        let mut third_level_entries_count =
            second_level_entries_count.div_ceil(HUGE_PAGE_SIZE) + 1;
        let forth_level_entries_count = third_level_entries_count
            .div_ceil(PAGE_DIRECTORY_ENTRIES)
            .clamp(1, 256);
        let mut next_mapped = unsafe { PhysicalAddress::new_unchecked(0) };
        for forth_entry in &mut self.entries[(PAGE_DIRECTORY_ENTRIES / 2)
            ..(forth_level_entries_count + (PAGE_DIRECTORY_ENTRIES / 2))]
        {
            let mut third_table =
                forth_entry.force_resolve_table_mut().unwrap();

            for third_entry in unsafe {
                &mut third_table.as_mut().entries[0
                    ..third_level_entries_count
                        .min(PAGE_DIRECTORY_ENTRIES)]
            } {
                let mut second_table =
                    third_entry.force_resolve_table_mut().unwrap();

                third_level_entries_count -= 1;
                for second_entry in unsafe {
                    &mut second_table.as_mut().entries[0
                        ..second_level_entries_count
                            .min(PAGE_DIRECTORY_ENTRIES)]
                } {
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

#[ext]
pub impl PageMap {
    /// Reallocates the page array on the buddy allocator.
    fn reallocate(init: &'static mut LateInit<PageMap>) {}
}
