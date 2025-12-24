use core::ptr;

use crate::{registers::cr3, structures::paging::PageTableEntry};
use common::{
    address_types::VirtualAddress,
    constants::{PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT},
    enums::{PageSize, PageTableLevel},
    error::EntryError,
};

// ANCHOR: page_table
#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}
// ANCHOR_END: page_table

#[derive(Debug)]
pub enum EntryIndex {
    Entry(&'static PageTableEntry),
    Index(usize),
    PageDoesNotFit,
    OutOfEntries,
}

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
    pub fn try_fetch_table(
        &'static self,
        start_at: usize,
        table_level: PageTableLevel,
        page_size: PageSize,
    ) -> EntryIndex {
        if !page_size.allocatable_at(table_level) {
            return EntryIndex::PageDoesNotFit;
        }

        for (i, entry) in self.entries.iter().enumerate().skip(start_at) {
            match entry.mapped_table() {
                Ok(_) => {
                    return EntryIndex::Entry(entry);
                }
                Err(EntryError::NoMapping) => {
                    return EntryIndex::Index(i);
                }
                Err(EntryError::NotATable) => continue,
            }
        }
        EntryIndex::OutOfEntries
    }
}
// ANCHOR_END: page_table_impl
