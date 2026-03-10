#[cfg(target_arch = "x86_64")]
use core::ptr::NonNull;

use crate::structures::paging::{PageEntryFlags, PageTable};
use common::{
    address_types::PhysicalAddress, constants::REGULAR_PAGE_ALIGNMENT,
    error::EntryError,
};
use macros::bitfields;

#[bitfields]
pub struct PageTableEntry {
    #[flag(rwc(0), flag_type = PageEntryFlags)]
    flags: B12,
    #[flag(rw, flag_type = PhysicalAddress, dont_shift)]
    address: B51,
    not_executable: B1,
}
// ANCHOR_END: page_table_entry

// ANCHOR: impl_page_table_entry
impl PageTableEntry {
    /// Map new frame with the given with the given flags.
    ///
    /// # Safety
    ///
    /// This function doesn't check if address is properly aligned, and if
    /// the entry was already mapped.
    // ANCHOR: page_table_entry_map_unchecked
    #[inline]
    pub unsafe fn map_unchecked(
        &mut self,
        frame: PhysicalAddress,
        flags: PageEntryFlags,
    ) {
        self.set_flags(flags.present());
        self.set_address(frame.as_usize() as u64);
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
    pub unsafe fn map(
        &mut self,
        frame: PhysicalAddress,
        flags: PageEntryFlags,
    ) {
        if !self.get_flags().is_present()
            && frame.is_aligned(REGULAR_PAGE_ALIGNMENT)
        {
            unsafe { self.map_unchecked(frame, flags) };
        }
    }
    // ANCHOR_END: page_table_entry_map

    /// Return the physical address that is mapped by this
    /// entry, if this entry is not mapped, return None.
    // ANCHOR: page_table_entry_mapped
    #[inline]
    pub fn mapped(&self) -> Result<PhysicalAddress, EntryError> {
        if self.get_flags().is_present() {
            Ok(self.get_address())
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
    // ANCHOR: page_table_entry_mapped_table
    #[cfg(target_arch = "x86_64")]
    pub fn mapped_table(&self) -> Result<NonNull<PageTable>, EntryError> {
        // first check if the entry is mapped.

        let pt = self.mapped()?.translate().as_non_null::<PageTable>();
        // then check if it is a table.

        let flags = self.get_flags();
        if !flags.is_huge_page() && flags.is_table() {
            Ok(pt)
        } else {
            Err(EntryError::NotATable)
        }
    }
    // ANCHOR_END: page_table_entry_mapped_table

    pub fn table_index(&self) -> usize {
        let table_offset = self as *const _ as usize & ((1 << 12) - 1);
        table_offset / size_of::<PageTableEntry>()
    }
}
