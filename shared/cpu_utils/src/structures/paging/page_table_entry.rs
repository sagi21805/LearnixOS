use common::{
    address_types::PhysicalAddress,
    constants::{ENTRY_ADDRESS_MASK, REGULAR_PAGE_ALIGNMENT},
    error::EntryError,
};

use crate::structures::paging::PageTable;

use super::PageEntryFlags;

// ANCHOR: page_table_entry
#[derive(Debug, Clone)]
pub struct PageTableEntry(pub u64);
// ANCHOR_END: page_table_entry

// ANCHOR: impl_page_table_entry
impl PageTableEntry {
    // ANCHOR: page_table_entry_flags
    table_entry_flags!();
    // ANCHOR_END: page_table_entry_flags

    // ANCHOR: page_table_entry_empty
    #[inline]
    pub(crate) const fn empty() -> Self {
        Self(0)
    }
    // ANCHOR_END: page_table_entry_empty

    /// Set all of the flags to zero.
    // ANCHOR: page_table_entry_reset_flags
    pub const fn reset_flags(&mut self) {
        self.0 &= ENTRY_ADDRESS_MASK;
    }
    // ANCHOR_END: page_table_entry_reset_flags

    /// Set the flags without a reset to previous flags.
    ///
    /// # Safety
    /// If there are some flags set prior to this, it will
    /// lead to undefined behavior
    // ANCHOR: page_table_entry_set_flags_unchecked
    pub const unsafe fn set_flags_unchecked(
        &mut self,
        flags: PageEntryFlags,
    ) {
        self.0 |= flags.0;
    }
    // ANCHOR_END: page_table_entry_set_flags_unchecked

    /// Set the flags of the entry
    // ANCHOR: page_table_entry_set_flags
    pub const fn set_flags(&mut self, flags: PageEntryFlags) {
        self.reset_flags();
        unsafe { self.set_flags_unchecked(flags) };
    }
    // ANCHOR_END: page_table_entry_set_flags

    // ANCHOR: page_table_entry_map_unchecked
    #[inline]
    pub const unsafe fn map_unchecked(
        &mut self,
        frame: PhysicalAddress,
        flags: PageEntryFlags,
    ) {
        *self = Self::empty();
        unsafe { self.set_flags_unchecked(flags) };
        self.set_present();
        // Set the new address
        self.0 |= frame.as_usize() as u64 & ENTRY_ADDRESS_MASK;
    }
    // ANCHOR_END: page_table_entry_map_unchecked

    /// Map a frame to the page table entry while checking
    /// flags and frame alignment but **not** the ownership
    /// of the frame address This function **will** set
    /// the entry as present even if it was not specified in
    /// the flags.
    ///
    /// # Parameters
    ///
    /// - `frame`: The physical address of the mapped frame
    ///
    /// # Interrupts
    /// This function will raise a PAGE_FAULT if the entry
    /// is already mapped
    ///
    /// # Safety
    /// The `frame` address should not be used by anyone
    /// except the corresponding virtual address,
    /// and should be marked owned by it in a memory
    /// allocator
    // ANCHOR: page_table_entry_map
    #[inline]
    pub const unsafe fn map(
        &mut self,
        frame: PhysicalAddress,
        flags: PageEntryFlags,
    ) {
        if !self.is_present() && frame.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            unsafe { self.map_unchecked(frame, flags) };
        }
    }
    // ANCHOR_END: page_table_entry_map

    // ANCHOR: page_table_entry_mapped_unchecked
    #[inline]
    pub const unsafe fn mapped_unchecked(&self) -> PhysicalAddress {
        unsafe {
            PhysicalAddress::new_unchecked(
                (self.0 & ENTRY_ADDRESS_MASK) as usize,
            )
        }
    }
    // ANCHOR_END: page_table_entry_mapped_unchecked

    /// Return the physical address that is mapped by this
    /// entry, if this entry is not mapped, return None.
    // ANCHOR: page_table_entry_mapped
    #[inline]
    pub fn mapped(&self) -> Result<PhysicalAddress, EntryError> {
        if self.is_present() {
            unsafe { Ok(self.mapped_unchecked()) }
        } else {
            Err(EntryError::NoMapping)
        }
    }
    // ANCHOR_END: page_table_entry_mapped

    /// Return the physical address mapped by this table as
    /// a reference into a page table.
    ///
    /// This method assumes all page tables are identity
    /// mapped.
    // ANCHOR: page_table_entry_mapped_table_mut
    #[cfg(target_arch = "x86_64")]
    pub fn mapped_table_mut(&self) -> Result<&mut PageTable, EntryError> {
        // first check if the entry is mapped.
        let pt = unsafe {
            &mut *self.mapped()?.translate().as_mut_ptr::<PageTable>()
        };
        // then check if it is a table.
        if !self.is_huge_page() && self.is_table() {
            Ok(pt)
        } else {
            Err(EntryError::NotATable)
        }
    }
    // ANCHOR_END: page_table_entry_mapped_table_mut

    // ANCHOR: page_table_entry_mapped_table
    #[cfg(target_arch = "x86_64")]
    pub fn mapped_table(&self) -> Result<&PageTable, EntryError> {
        // first check if the entry is mapped.
        let pt =
            unsafe { &*self.mapped()?.translate().as_ptr::<PageTable>() };
        // then check if it is a table.
        if !self.is_huge_page() && self.is_table() {
            Ok(pt)
        } else {
            Err(EntryError::NotATable)
        }
    }
    // ANCHOR_END: page_table_entry_mapped_table
}
