use core::ptr::{self, NonNull};

use crate::{
    registers::cr3,
    structures::paging::{PageEntryFlags, PageTableEntry},
};
use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    constants::{PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, MappingError},
};

use extend;

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}

#[derive(Debug)]
pub enum EntryIndex {
    Entry(&'static PageTableEntry),
    Index(usize),
    PageDoesNotFit,
    OutOfEntries,
}

impl PageTable {
    /// Create an empty page table
    #[inline]
    pub const fn empty() -> Self {
        Self {
            entries: { [PageTableEntry::new(); PAGE_DIRECTORY_ENTRIES] },
        }
    }

    /// Create an empty page table at the given virtual address
    ///
    /// # Safety
    /// This function works on every address, and will override the data at
    /// that address
    #[inline]
    pub unsafe fn empty_from_ptr(
        page_table_ptr: VirtualAddress,
    ) -> Option<NonNull<PageTable>> {
        if !page_table_ptr.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            return None;
        }
        unsafe {
            ptr::write_volatile(
                page_table_ptr.as_non_null::<PageTable>().as_ptr(),
                PageTable::empty(),
            );
            Some(page_table_ptr.as_non_null::<PageTable>())
        }
    }

    #[inline]
    pub fn current_table() -> NonNull<PageTable> {
        NonNull::new(cr3::read() as usize as *mut PageTable)
            .expect("Page table pointer is not present in cr3, found NULL")
    }

    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub fn address(&self) -> VirtualAddress {
        unsafe {
            VirtualAddress::new_unchecked(self as *const Self as usize)
        }
    }

    /// Tries to fetch a page table entry or an empty page starting from
    /// the given index.
    ///
    /// Returns the index of the found entry and the page table if found.
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

#[extend::ext]
pub impl VirtualAddress {
    #[cfg(target_arch = "x86_64")]
    fn walk(&self) -> impl Iterator<Item = NonNull<PageTableEntry>> {
        let mut table = Some(PageTable::current_table());
        let mut level = Some(PageTableLevel::PML4);
        ::core::iter::from_fn(move || {
            let entry =
                unsafe { &table?.as_ref().entries[self.index_of(level?)] };

            if entry.get_flags().is_present() {
                table = entry.mapped_table().ok();
                level = level?.next();
                Some(NonNull::from_ref(entry))
            } else {
                None
            }
        })
    }

    #[cfg(target_arch = "x86_64")]
    fn map(
        &self,
        address: PhysicalAddress,
        flags: Option<PageEntryFlags>,
        page_size: PageSize,
    ) -> Result<(), MappingError> {
        let mut entry = self
            .walk()
            .nth(page_size.mapping_table() as usize)
            .ok_or(MappingError::TableDoesNotExist)?;
        unsafe {
            entry.as_mut().map(
                address,
                flags.unwrap_or(PageEntryFlags::regular_page_flags()),
            )
        }
        Ok(())
    }
}
