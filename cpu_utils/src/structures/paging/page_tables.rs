/// This module contains very basic code that helps to interface and create initial page table
///
/// The more advanced code that will be used in the future to allocate table will be in the kernel
///
/// --------------------------------------------------------------------------------------------------
use super::address_types::{PhysicalAddress, VirtualAddress};
use crate::flag;
use crate::registers::cr3::cr3_read;
use common::constants::enums::{PageSize, PageTableLevel};
use common::constants::error::{EntryError, TableError};
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
    /// Returns flags suitable for a regular present and writable page entry.
    ///
    /// The returned flags indicate that the page is present and writable, but do not set any special attributes such as huge page or table.
    ///
    /// # Examples
    ///
    /// ```
    /// let flags = PageEntryFlags::regular_page_flags();
    /// assert!(flags.as_u64() & PageEntryFlags::PRESENT != 0);
    /// assert!(flags.as_u64() & PageEntryFlags::WRITABLE != 0);
    /// ```
    pub const fn regular_page_flags() -> Self {
        PageEntryFlags::new()
            .set_chain_present()
            .set_chain_writable()
    }
    table_entry_flags!();
    /// Returns the raw 64-bit value of the page entry or flags.
    ///
    /// # Examples
    ///
    /// ```
    /// let flags = PageEntryFlags::regular_page_flags();
    /// let raw = flags.as_u64();
    /// assert_eq!(raw & 0b11, 0b11); // present and writable bits set
    /// ```
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Returns a new instance with all flags cleared.
    pub(crate) const fn empty() -> Self {
        Self(0)
    }

    table_entry_flags!();

    /// Sets the page table entry flags, preserving the address bits.
    ///
    /// Replaces any existing flags with the provided `flags`, while leaving the mapped address unchanged.
    pub const fn set_flags(&mut self, flags: PageEntryFlags) {
        self.0 &= ADDRESS_MASK; // zero out all previous flags.
        self.0 |= flags.as_u64(); // set new flags;
    }

    /// Maps a physical frame to this page table entry if it is not already present and the frame is properly aligned.
    ///
    /// Sets the entry as present regardless of the provided flags. Raises a page fault if the entry is already mapped.
    ///
    /// # Parameters
    ///
    /// - `frame`: Physical address of the frame to map. Must be aligned to the regular page size.
    /// - `flags`: Flags to set for the entry (the present flag will be enforced).
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `frame` is exclusively owned by the corresponding virtual address and properly tracked by the memory allocator.
    pub const fn map(&mut self, frame: PhysicalAddress, flags: PageEntryFlags) {
        if !self.present() && frame.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            self.map_unchecked(frame, flags);
        }
    }

    /// Maps a physical frame to this page table entry without performing any checks.
    ///
    /// Overwrites the entry's address and flags with the provided values, marking it as present. No validation is performed on the frame alignment or existing entry state. Intended for use in contexts where safety is guaranteed externally.
    pub const fn map_unchecked(&mut self, frame: PhysicalAddress, flags: PageEntryFlags) {
        self.set_flags(flags);
        self.set_present();
        self.0 &= !ADDRESS_MASK; // Zero out the address part of the entry.
        self.0 |= frame.as_usize() as u64 & ADDRESS_MASK; // Set the new address
    }

    /// Returns the physical address mapped by this entry if it is present.
    ///
    /// Returns an error if the entry does not map a physical address.
    #[inline]
    pub fn mapped(&self) -> Result<PhysicalAddress, EntryError> {
        if self.present() {
            unsafe { Ok(self.mapped_unchecked()) }
        } else {
            Err(EntryError::NoMapping)
        }
    }

    /// Returns the physical address mapped by this entry without checking if the entry is present.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the entry is valid and actually maps a physical address. Calling this on an unmapped or invalid entry may result in undefined behavior.
    #[inline]
    pub const unsafe fn mapped_unchecked(&self) -> PhysicalAddress {
        unsafe { PhysicalAddress::new_unchecked((self.0 & ADDRESS_MASK) as usize) }
    }

    /// Returns a mutable reference to the page table mapped by this entry.
    ///
    /// The entry must be present, not a huge page, and marked as a table. Returns an error if the entry is not mapped or does not reference a page table. Assumes all page tables are identity mapped.
    ///
    /// # Returns
    ///
    /// - `Ok(&mut PageTable)` if the entry maps to a valid page table.
    /// - `Err(EntryError::NoMapping)` if the entry is not present.
    /// - `Err(EntryError::NotATable)` if the entry does not reference a page table.
    pub fn mapped_table_mut(&self) -> Result<&mut PageTable, EntryError> {
        // first check if the entry is mapped.
        let table = unsafe { &mut *self.mapped()?.as_mut_ptr::<PageTable>() };
        // then check if it is a table.
        if !self.huge_page() && self.is_table() {
            Ok(table)
        } else {
            Err(EntryError::NotATable)
        }
    }

    /// Returns a reference to the page table mapped by this entry if it is a table and not a huge page.
    ///
    /// Returns an error if the entry is not mapped or does not point to a page table.
    pub fn mapped_table(&self) -> Result<&PageTable, EntryError> {
        // first check if the entry is mapped.
        let table = unsafe { &*self.mapped()?.as_ptr::<PageTable>() };
        // then check if it is a table.
        if !self.huge_page() && self.is_table() {
            Ok(table)
        } else {
            Err(EntryError::NotATable)
        }
    }

    /// Returns the raw 64-bit value of the page entry or flags.
    ///
    /// # Examples
    ///
    /// ```
    /// let flags = PageEntryFlags::regular_page_flags();
    /// let raw = flags.as_u64();
    /// assert_eq!(raw & 0b11, 0b11); // present and writable bits set
    /// ```
    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}

impl PageTable {
    /// Returns a mutable reference to a `PageTable` at the given memory address.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `page_table_ptr` is valid, properly aligned, and points to a region of memory that is safe to interpret as a `PageTable`. Using an invalid pointer may cause undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// let ptr = 0x1000usize; // Must be a valid, aligned page table address
    /// let table = unsafe { PageTable::from_ptr(ptr) };
    /// ```
    #[inline]
    pub const unsafe fn from_ptr(page_table_ptr: usize) -> &'static mut PageTable {
        unsafe { &mut *(page_table_ptr as *mut PageTable) }
    }

    /// Writes an empty page table to the specified memory address and returns a mutable reference to it.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `page_table_ptr` is valid, properly aligned, and points to writable memory large enough for a `PageTable`.
    ///
    /// # Examples
    ///
    /// ```
    /// let page_table_addr = 0x1000usize; // Must be a valid, aligned address
    /// let table = unsafe { PageTable::empty_from_ptr(page_table_addr) };
    /// assert!(table.entries.iter().all(|e| e.as_u64() == 0));
    /// ```
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

    /// Returns the virtual address of this page table.
    ///
    /// # Examples
    ///
    /// ```
    /// let table = PageTable::empty();
    /// let addr = table.address();
    /// assert_eq!(addr.as_usize(), &table as *const _ as usize);
    /// ```
    #[inline]
    pub fn address(&self) -> VirtualAddress {
        unsafe { VirtualAddress::new_unchecked(self as *const Self as usize) }
    }

    /// Searches for the next mapped page table or empty entry starting from a given index.
    ///
    /// Iterates over page table entries beginning at `start_at`, returning the index and a reference to the mapped table if found. If an empty entry is encountered, returns its index and `None`. Skips entries that are not tables or are huge pages. If no suitable entry is found, returns `(PAGE_DIRECTORY_ENTRIES, None)`.
    ///
    /// # Parameters
    /// - `start_at`: The index to begin searching from within the entries.
    /// - `table_level`: The current level of the page table hierarchy.
    /// - `page_size`: The page size being considered.
    ///
    /// # Returns
    /// A tuple containing the index of the found entry and an optional reference to the mapped page table. If no entry is found, the index will be `PAGE_DIRECTORY_ENTRIES` and the reference will be `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let (idx, table_opt) = page_table.fetch_table_or_empty(0, &table_level, &page_size);
    /// if let Some(table) = table_opt {
    ///     // Found a mapped table at index `idx`
    /// } else {
    ///     // Found an empty entry at index `idx`
    /// }
    /// ```
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
                _ => unreachable!(),
            }
        }
        (PAGE_DIRECTORY_ENTRIES, None)
    }

    /// Returns a reference to the current top-level page table by reading the CR3 register.
    ///
    /// # Safety
    ///
    /// The returned reference assumes that the address in CR3 is valid and mapped for the lifetime of the program.
    ///
    /// # Examples
    ///
    /// ```
    /// let table = PageTable::current_table();
    /// // Use `table` to inspect or traverse the current page table hierarchy.
    /// ```
    pub fn current_table() -> &'static PageTable {
        unsafe { core::mem::transmute(cr3_read()) }
    }

    /// Returns a mutable reference to the current top-level page table by reading the CR3 register.
    ///
    /// # Safety
    ///
    /// This function uses an unsafe transmute to cast the physical address from CR3 to a mutable reference.
    /// The caller must ensure that this reference is valid and not aliased elsewhere.
    ///
    /// # Examples
    ///
    /// ```
    /// let table = PageTable::current_table_mut();
    /// // Now you can modify the current page table entries.
    /// ```
    pub fn current_table_mut() -> &'static mut PageTable {
        unsafe { core::mem::transmute(cr3_read()) }
    }

    /// Searches the page table hierarchy for the first available virtual page of the specified size.
    ///
    /// Traverses the page tables from the fourth level down, returning the virtual address of the first unmapped page suitable for the given `page_size`. Returns an error if no available page is found.
    ///
    /// # Returns
    /// - `Ok(VirtualAddress)` with the address of the first available page.
    /// - `Err(TableError)` if no suitable page is available.
    ///
    /// # Examples
    ///
    /// ```
    /// let page = PageTable::find_available_page(PageSize::Regular).unwrap();
    /// assert!(page.is_aligned(PageSize::Regular));
    /// ```
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
