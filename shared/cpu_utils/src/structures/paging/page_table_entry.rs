use common::{
    address_types::PhysicalAddress,
    constants::{ENTRY_ADDRESS_MASK, REGULAR_PAGE_ALIGNMENT},
    error::EntryError,
};

use crate::structures::paging::PageTable;

use super::PageEntryFlags;

#[derive(Debug, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    table_entry_flags!();

    #[inline]
    pub(crate) const fn empty() -> Self {
        Self(0)
    }

    /// Set all of the flags to zero.
    pub const fn reset_flags(&mut self) {
        self.0 &= ENTRY_ADDRESS_MASK;
    }

    /// Set the flags without a reset to previous flags.
    ///
    /// # Safety
    /// If there are some flags set prior to this, it will lead to undefined behavior
    pub const unsafe fn set_flags_unchecked(&mut self, flags: PageEntryFlags) {
        self.0 |= flags.as_u64()
    }

    /// Set the flags of the entry
    pub const fn set_flags(&mut self, flags: PageEntryFlags) {
        self.reset_flags();
        unsafe { self.set_flags_unchecked(flags) };
    }

    pub const unsafe fn map_unchecked(&mut self, frame: PhysicalAddress, flags: PageEntryFlags) {
        *self = Self::empty();
        unsafe { self.set_flags_unchecked(flags) };
        self.set_present();
        self.0 |= frame.as_usize() as u64 & ENTRY_ADDRESS_MASK; // Set the new address
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
    pub const unsafe fn map(&mut self, frame: PhysicalAddress, flags: PageEntryFlags) {
        if !self.is_present() && frame.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            unsafe { self.map_unchecked(frame, flags) };
        }
    }

    #[inline]
    pub const unsafe fn mapped_unchecked(&self) -> PhysicalAddress {
        unsafe { PhysicalAddress::new_unchecked((self.0 & ENTRY_ADDRESS_MASK) as usize) }
    }

    #[inline]
    /// Return the physical address that is mapped by this entry, if this entry is not mapped, return None.
    pub fn mapped(&self) -> Result<PhysicalAddress, EntryError> {
        if self.is_present() {
            unsafe { Ok(self.mapped_unchecked()) }
        } else {
            Err(EntryError::NoMapping)
        }
    }

    #[cfg(target_arch = "x86_64")]
    /// Return the physical address mapped by this table as a reference into a page table.
    ///
    /// This method assumes all page tables are identity mapped.
    pub fn mapped_table_mut(&self) -> Result<&mut PageTable, EntryError> {
        // first check if the entry is mapped.
        let table = unsafe { &mut *self.mapped()?.translate().as_mut_ptr::<PageTable>() };
        // then check if it is a table.
        if !self.is_huge_page() && self.is_table() {
            Ok(table)
        } else {
            Err(EntryError::NotATable)
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn mapped_table(&self) -> Result<&PageTable, EntryError> {
        // first check if the entry is mapped.
        let table = unsafe { &*self.mapped()?.translate().as_ptr::<PageTable>() };
        // then check if it is a table.
        if self.is_huge_page() && self.is_table() {
            Ok(table)
        } else {
            Err(EntryError::NotATable)
        }
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
