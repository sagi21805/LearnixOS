/// This module contains very basic code that helps to interface and create initial page table
///
/// The more advanced code that will be used in the future to allocate table will be in the kernel
///
/// --------------------------------------------------------------------------------------------------
use super::super::super::registers::get_current_page_table;
use super::address_types::{PhysicalAddress, VirtualAddress};
use crate::flag;
use constants::enums::PageSize;
use constants::values::{PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT};

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

    flag!(full, 9);

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
    pub const fn mapped_address(&self) -> Option<PhysicalAddress> {
        if self.present() {
            unsafe { Some(self.mapped_address_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub const unsafe fn mapped_address_unchecked(&self) -> PhysicalAddress {
        PhysicalAddress::new((self.0 & 0x0000_fffffffff_000) as usize)
    }

    #[inline]
    /// Return the physical address mapped by this table as a reference into a page table.
    ///
    /// This method assumes all page tables are identity mapped.
    pub unsafe fn as_table_mut(&self) -> &mut PageTable {
        if !self.huge_page() {
            match self.mapped_address() {
                Some(mapped_address) => unsafe {
                    core::mem::transmute::<PhysicalAddress, &mut PageTable>(mapped_address)
                },
                None => {
                    panic!("Page table is not mapped, page fault");
                }
            }
        } else {
            panic!("The page is authoritative so there is no next table");
        }
    }

    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn as_table_mut_unchecked(&self) -> &mut PageTable {
        core::mem::transmute::<PhysicalAddress, &mut PageTable>(self.mapped_address_unchecked())
    }

    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn as_table_unchecked(&self) -> &PageTable {
        core::mem::transmute::<PhysicalAddress, &PageTable>(self.mapped_address_unchecked())
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

    #[inline]
    pub fn find_available_page(&self, page_size: PageSize) -> Option<VirtualAddress> {
        for (i4, forth_entry) in self.entries.iter().enumerate() {
            if !forth_entry.present() {
                continue;
            }

            let third_table = unsafe { forth_entry.as_table_unchecked() };

            for (i3, third_entry) in third_table.entries.iter().enumerate() {
                if !third_entry.present() {
                    return Some(VirtualAddress::from_indexes(i4, i3, 0, 0));
                }

                if third_entry.huge_page()
                    || !matches!(page_size, PageSize::Big | PageSize::Regular)
                {
                    continue;
                }

                let second_table = unsafe { third_entry.as_table_unchecked() };

                for (i2, second_entry) in second_table.entries.iter().enumerate() {
                    if !second_entry.present() {
                        return Some(VirtualAddress::from_indexes(i4, i3, i2, 0));
                    }

                    if second_entry.huge_page() || !matches!(page_size, PageSize::Regular) {
                        continue;
                    }

                    let first_table = unsafe { second_entry.as_table_unchecked() };

                    for (i1, first_entry) in first_table.entries.iter().enumerate() {
                        if !first_entry.present() {
                            return Some(VirtualAddress::from_indexes(i4, i3, i2, i1));
                        }
                    }
                }
            }
        }
        None
    }
}
