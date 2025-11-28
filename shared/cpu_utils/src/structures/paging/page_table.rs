use core::ptr;

use crate::{registers::cr3, structures::paging::PageTableEntry};
use common::{
    address_types::VirtualAddress,
    constants::{PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, TableError},
};

// ANCHOR: page_table
#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}
// ANCHOR_END: page_table

// ANCHOR: page_table_impl
impl PageTable {
    // ANCHOR: page_table_empty
    /// Create an empty page table
    #[inline]
    pub const fn empty() -> Self {
        Self {
            entries: {
                [const { PageTableEntry::empty() }; PAGE_DIRECTORY_ENTRIES]
            },
        }
    }
    // ANCHOR_END: page_table_empty

    /// Create an empty page table at the given virtual address
    ///
    /// # Safety
    /// This function works on every address, and will override the data at
    /// that address
    // ANCHOR: page_table_empty_from_ptr
    #[inline]
    pub unsafe fn empty_from_ptr(
        page_table_ptr: VirtualAddress,
    ) -> Option<&'static mut PageTable> {
        if !page_table_ptr.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            return None;
        }
        unsafe {
            ptr::write_volatile(
                page_table_ptr.as_mut_ptr::<PageTable>(),
                PageTable::empty(),
            );
            Some(&mut *page_table_ptr.as_mut_ptr::<PageTable>())
        }
    }
    // ANCHOR_END: page_table_empty_from_ptr

    // ANCHOR: page_table_current_table
    #[inline]
    pub fn current_table() -> &'static PageTable {
        unsafe {
            &*core::ptr::with_exposed_provenance(cr3::read() as usize)
        }
    }

    #[inline]
    pub fn current_table_mut() -> &'static mut PageTable {
        unsafe {
            &mut *core::ptr::with_exposed_provenance_mut(
                cr3::read() as usize
            )
        }
    }
    // ANCHOR_END: page_table_current_table

    // ANCHOR: page_table_address
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub fn address(&self) -> VirtualAddress {
        unsafe {
            VirtualAddress::new_unchecked(self as *const Self as usize)
        }
    }
    // ANCHOR_END: page_table_address

    /// Tries to fetch a page table entry or an empty page starting from
    /// the given index.
    ///
    /// Returns the index of the found entry and the page table if found.
    // Anchor: page_table_try_fetch_table
    #[cfg(target_arch = "x86_64")]
    fn try_fetch_table(
        &self,
        start_at: usize,
        table_level: PageTableLevel,
        page_size: PageSize,
    ) -> (usize, Option<&PageTable>) {
        for (i, entry) in self.entries.iter().enumerate().skip(start_at) {
            match entry.mapped_table() {
                Ok(v) => {
                    if page_size.exceeds(table_level) {
                        continue;
                    }
                    return (i, Some(v));
                }
                Err(EntryError::NoMapping) => {
                    return (i, None);
                }
                Err(EntryError::NotATable) => continue,
            }
        }
        (PAGE_DIRECTORY_ENTRIES, None)
    }

    /// Find an avavilable page in the given size.
    // ANCHOR: page_table_find_available_page
    #[cfg(target_arch = "x86_64")]
    pub fn find_available_page(
        page_size: PageSize,
    ) -> Result<VirtualAddress, TableError> {
        const LEVELS: usize = 4;
        let mut level_indices = [0usize; LEVELS];
        let mut page_tables = [Self::current_table(); LEVELS];
        let mut current_level = PageTableLevel::PML4;
        loop {
            let current_table = page_tables[current_level as usize];

            let next_table = match current_table.try_fetch_table(
                level_indices[current_level as usize],
                current_level,
                page_size,
            ) {
                (PAGE_DIRECTORY_ENTRIES, None) => {
                    current_level = current_level.prev()?;
                    level_indices[current_level as usize] += 1;
                    continue;
                }
                (i, Some(table)) => {
                    level_indices[current_level as usize] = i;
                    table
                }
                (i, None) => {
                    level_indices[current_level as usize] = i;
                    return Ok(VirtualAddress::from_indices(
                        level_indices,
                    ));
                }
            };
            let next_level = current_level
                .next()
                .expect("Can't go next on a first level table");
            page_tables[next_level as usize] = next_table;
            level_indices[next_level as usize] += 1;
        }
    }
    // ANCHOR_END: page_table_find_available_page
}
// ANCHOR_END: page_table_impl
