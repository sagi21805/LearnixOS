use core::ptr;

use crate::{registers::cr3::cr3_read, structures::paging::PageTableEntry};
use common::{
    address_types::VirtualAddress,
    constants::{PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, TableError},
};

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}

impl PageTable {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            entries: [const { PageTableEntry::empty() }; PAGE_DIRECTORY_ENTRIES],
        }
    }
    #[inline]
    pub unsafe fn empty_from_ptr(page_table_ptr: VirtualAddress) -> Option<&'static mut PageTable> {
        if !page_table_ptr.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            return None;
        }
        unsafe {
            ptr::write_volatile(page_table_ptr.as_mut_ptr::<PageTable>(), PageTable::empty());
            return Some(&mut *page_table_ptr.as_mut_ptr::<PageTable>());
        }
    }

    pub fn current_table() -> &'static PageTable {
        unsafe { core::mem::transmute(cr3_read()) }
    }

    pub fn current_table_mut() -> &'static mut PageTable {
        unsafe { core::mem::transmute(cr3_read()) }
    }

    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub fn address(&self) -> VirtualAddress {
        unsafe { VirtualAddress::new_unchecked(self as *const Self as usize) }
    }

    #[cfg(target_arch = "x86_64")]
    fn fetch_table_or_empty(
        &self,
        start_at: usize,
        table_level: &PageTableLevel,
        page_size: &PageSize,
    ) -> (usize, Option<&PageTable>) {
        for (i, entry) in self.entries.iter().enumerate().skip(start_at) {
            match entry.mapped_table() {
                Ok(v) => {
                    if page_size.exceeds(table_level) {
                        continue;
                    }
                    return (i, Some(v));
                }
                Err(EntryError::NoMapping) => return (i, None),
                Err(EntryError::NotATable) => continue,
            }
        }
        (PAGE_DIRECTORY_ENTRIES, None)
    }

    /// Find an avavilable page.
    #[cfg(target_arch = "x86_64")]
    pub fn find_available_page(page_size: PageSize) -> Result<VirtualAddress, TableError> {
        const LEVELS: usize = 4;
        let mut level_indices = [0usize; LEVELS];
        let mut page_tables = [Self::current_table(); LEVELS];
        let mut current_level = PageTableLevel::ForthLevel;
        loop {
            let current_table = page_tables[current_level.as_usize()];

            let next_table = match current_table.fetch_table_or_empty(
                level_indices[current_level.as_usize()],
                &current_level,
                &page_size,
            ) {
                (PAGE_DIRECTORY_ENTRIES, None) => {
                    current_level = current_level.prev()?;
                    level_indices[current_level.as_usize()] += 1;
                    continue;
                }
                (i, Some(table)) => {
                    level_indices[current_level.as_usize()] = i;
                    table
                }
                (i, None) => {
                    level_indices[current_level.as_usize()] = i;
                    return Ok(VirtualAddress::from_indices(level_indices));
                }
            };
            let next_level = current_level
                .next()
                .expect("Can't go next on a first level table");
            page_tables[next_level.as_usize()] = next_table;
            level_indices[next_level.as_usize()] += 1;
        }
    }
}
