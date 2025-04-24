/// This module contains very basic code that helps to interface and create initial page table
/// 
/// The more advanced code that will be used in the future to allocate table will be in the kernel
/// 
/// --------------------------------------------------------------------------------------------------

use super::address_types::{PhysicalAddress, VirtualAddress};
use constants::values::{PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT};
use crate::flag;

#[derive(Debug)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    
    #[inline]
    pub(crate) const fn empty() -> Self {
        Self(0)
    }

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

    // This page is holding data and is not executable
    flag!(not_executable, 63);

    #[inline]
    /// Map a frame to the page table entry while checking flags and frame alignment but **not** the ownership of the frame address
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
    pub const unsafe fn map_unchecked(&mut self, frame: PhysicalAddress) {
        if !self.present() && frame.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            self.0 |= frame.as_usize() as u64 & 0xfffffffff_000;
            self.set_present();
            self.set_writable();
        } else {
            todo!(
                "Page is already mapped, raise a page fault when interrupt descriptor table is initialized"
            );
        }
    }

    #[inline]
    /// Return the physical address that is mapped by this entry
    /// 
    /// # Interrupts
    /// This function will raise a PAGE_FAULT if it doesn't map any entry
    pub(crate) const fn mapped_address(&self) -> PhysicalAddress {
        if self.present() {
            PhysicalAddress::new((self.0 & 0x0000_fffffffff_000) as usize)
        } else {
            panic!("Page does not mapped to any address");
        }
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[repr(C)]
#[repr(align(4096))]
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
    pub fn address(&self) -> VirtualAddress {
        VirtualAddress::new(self as *const Self as usize)
    }

    // #[inline]
    // pub fn find_available_page(&self, page_size: PageSize) -> (PhysicalAddress, VirtualAddress) {
    //     // todo!();
    //     let mut table = unsafe { get_current_page_table() };

    //     for table_number in ((page_size.clone() as usize + 1)..=4).rev() {
    //         if table.entries[self.nth_pt_index(table_number)].present() {
    //             table = table.entries[self.nth_pt_index(table_number)].get_next_table_mut()
    //         } else {
    //             table.entries[self.nth_pt_index(table_number)].create_new_table();
    //         }
    //     }
    //     table.entries[self.nth_pt_index(page_size as usize)].map(address);
    // }
}
