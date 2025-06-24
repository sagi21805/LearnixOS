/// This module contains very basic code that helps to interface and create initial page table
///
/// The more advanced code that will be used in the future to allocate table will be in the kernel
///
/// --------------------------------------------------------------------------------------------------
use super::address_types::{PhysicalAddress, VirtualAddress};
use super::error::{EntryError, TableError};
use crate::flag;
use crate::registers::cr3::cr3_read;
use common::constants::enums::PageSize;
use common::constants::values::{
    PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
};
use core::ptr;
const ADDRESS_MASK: u64 = 0x0000_fffffffff_000;

macro_rules! table_entry_flags {
    () => {
        // Is this page present?
        flag!(present, 0);

        // Is this page writable?
        flag!(writable, 1);

        // Can this page be accessed from user mode
        flag!(usr_access, 2);

        // Writes go directly to memory
        flag!(write_through_cache, 3);

        // Disable cache for this page
        flag!(disable_cache, 4);

        // This flag can help identifying if an entry is the last one, or it is pointing to another directory
        // Is this page points to a custom memory address and not a page table?
        flag!(huge_page, 7);

        // Page isn’t flushed from caches on address space switch (PGE bit of CR4 register must be set)
        flag!(global, 8);

        // mark a table as full
        flag!(full, 9);

        // This entry points to a table
        flag!(is_table, 10);

        // This entry is at the top of the heirarchy.
        flag!(root_entry, 11);

        // This page is holding data and is not executable
        flag!(not_executable, 63);
    };
}

// Just a wrapper for the flags for easier use
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageEntryFlags(u64);

impl PageEntryFlags {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }
    pub const fn table_flags() -> Self {
        PageEntryFlags::new()
            .set_chain_present()
            .set_chain_writable()
            .set_chain_is_table()
    }
    pub const fn huge_page_flags() -> Self {
        PageEntryFlags::new()
            .set_chain_present()
            .set_chain_writable()
            .set_chain_huge_page()
    }
    pub const fn regular_page_flags() -> Self {
        PageEntryFlags::new()
            .set_chain_present()
            .set_chain_writable()
    }
    table_entry_flags!();
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    #[inline]
    pub(crate) const fn empty() -> Self {
        Self(0)
    }

    table_entry_flags!();

    pub const fn set_flags(&mut self, flags: PageEntryFlags) {
        self.0 &= ADDRESS_MASK; // zero out all previous flags.
        self.0 |= flags.as_u64(); // set new flags;
    }

    #[inline]
    /// Map a frame to the page table entry while checking flags and frame alignment but **not** the ownership of the frame address
    /// This function **will** set the entry as present even if it was not specified in the flags.
    ///
    /// # Parameters
    ///
    /// - `frame`: The physical address of the mapped frame
    ///
    /// # Interrupts
    /// This function will raise a PAGE_FAULT if the entry is already mapped
    ///
    /// # Safety
    /// The `frame` address should not be used by anyone except the corresponding virtual address,
    /// and should be marked owned by it in a memory allocator
    pub const fn map(&mut self, frame: PhysicalAddress, flags: PageEntryFlags) {
        if !self.present() && frame.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            self.set_flags(flags);
            self.set_present();
            self.0 |= frame.as_usize() as u64 & ADDRESS_MASK;
        } else {
            // todo!(
            // "Page is already mapped, raise a page fault when interrupt descriptor table is initialized"
            // );
        }
    }

    #[inline]
    /// Return the physical address that is mapped by this entry, if this entry is not mapped, return None.
    pub fn mapped(&self) -> Result<PhysicalAddress, EntryError> {
        if self.present() {
            unsafe { Ok(self.mapped_unchecked()) }
        } else {
            Err(EntryError::NoMapping(self.index_in_table()))
        }
    }

    #[inline]
    pub const unsafe fn mapped_unchecked(&self) -> PhysicalAddress {
        unsafe { PhysicalAddress::new_unchecked((self.0 & ADDRESS_MASK) as usize) }
    }

    #[inline]
    /// Return the physical address mapped by this table as a reference into a page table.
    ///
    /// This method assumes all page tables are identity mapped.
    pub fn mapped_table_mut(&mut self) -> Result<(usize, &mut PageTable), EntryError> {
        // first check if the entry is mapped.
        let table = unsafe { &mut *self.mapped()?.as_mut_ptr::<PageTable>() };
        // then check if it is a table.
        if !self.huge_page() && self.is_table() {
            Ok((self.index_in_table(), table))
        } else {
            Err(EntryError::NotATable(self.index_in_table()))
        }
    }

    pub fn mapped_table(&self) -> Result<(usize, &PageTable), EntryError> {
        // first check if the entry is mapped.
        let table = unsafe { &*self.mapped()?.as_ptr::<PageTable>() };
        // then check if it is a table.
        if !self.huge_page() && self.is_table() {
            Ok((self.index_in_table(), table))
        } else {
            Err(EntryError::NotATable(self.index_in_table()))
        }
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Return a reference to the parent page table of this entry, this is a physical address meant to be accessed with the identity page table
    pub(self) fn index_in_table(&self) -> usize {
        ((self as *const Self as usize) & (REGULAR_PAGE_SIZE - 1)) / size_of::<PageTableEntry>()
    }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}

impl PageTable {
    #[inline]
    pub const unsafe fn from_ptr(page_table_ptr: usize) -> &'static mut PageTable {
        unsafe { &mut *(page_table_ptr as *mut PageTable) }
    }

    #[inline]
    pub unsafe fn empty_from_ptr(page_table_ptr: usize) -> &'static mut PageTable {
        unsafe {
            ptr::write_volatile(page_table_ptr as *mut PageTable, PageTable::empty());
            &mut *(page_table_ptr as *mut PageTable)
        }
    }

    #[inline]
    pub const fn empty() -> Self {
        Self {
            entries: [const { PageTableEntry::empty() }; PAGE_DIRECTORY_ENTRIES],
        }
    }

    #[inline]
    pub fn address(&self) -> VirtualAddress {
        unsafe { VirtualAddress::new_unchecked(self as *const Self as usize) }
    }

    fn fetch_table_or_empty(
        &self,
        start_at: usize,
        table_level: usize,
        page_size: &PageSize,
    ) -> Result<(usize, &PageTable), EntryError> {
        for entry in self.entries.iter().skip(start_at) {
            match entry.mapped_table() {
                Ok(v) => {
                    if page_size.exceeds(table_level) {
                        continue;
                    }
                    return Ok(v);
                }
                Err(EntryError::NoMapping(v)) => return Err(EntryError::NoMapping(v)),
                Err(EntryError::NotATable(_)) => continue,
                _ => unreachable!(),
            }
        }
        Err(EntryError::Full)
    }

    pub fn current_table() -> &'static PageTable {
        unsafe { core::mem::transmute(cr3_read()) }
    }

    pub fn current_table_mut() -> &'static mut PageTable {
        unsafe { core::mem::transmute(cr3_read()) }
    }

    /// Find an avavilable page.
    pub fn find_contiguous_pages_in_current_table(
        page_size: PageSize,
    ) -> Result<VirtualAddress, TableError> {
        const LEVELS: usize = 4;
        let mut level_indices = [0usize; LEVELS];
        let mut page_tables = [Self::current_table(); LEVELS];
        let mut current_level = 0;
        loop {
            let current_table = page_tables[current_level];
            page_tables[current_level + 1] = match current_table.fetch_table_or_empty(
                level_indices[current_level],
                current_level,
                &page_size,
            ) {
                Ok((i, table)) => {
                    level_indices[current_level] = i;
                    table
                }
                Err(EntryError::NoMapping(i)) => {
                    level_indices[current_level] = i;
                    return Ok(VirtualAddress::from_indices(level_indices));
                }
                Err(EntryError::Full) => {
                    if current_level == 0 {
                        return Err(TableError::Full);
                    }
                    current_level -= 1;
                    level_indices[current_level] += 1;
                    continue;
                }
                _ => unreachable!(),
            };
            current_level += 1;
        }
    }
}
