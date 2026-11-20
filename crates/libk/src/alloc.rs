extern crate alloc;

use core::{alloc::Layout, ptr::NonNull};

use alloc::alloc::{AllocError, alloc_zeroed};

use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, MappingError},
};
use x86::structures::paging::{
    PageEntryFlags, PageTable, PageTableEntry, VirtualAddressExt,
};

use crate::println;

/// Allocates a zeroed table, and points `entry` at its physical address.
pub fn alloc_table(
    entry: &mut PageTableEntry,
) -> Result<NonNull<PageTable>, AllocError> {
    let ptr = unsafe { alloc_zeroed(Layout::new::<PageTable>()) };
    let table = NonNull::new(ptr).ok_or(AllocError)?.cast::<PageTable>();
    let address = VirtualAddress::from(ptr.addr())
        .translate()
        .expect("the global allocator returns mapped memory");
    unsafe {
        entry.map(address, PageEntryFlags::table_flags());
    }
    Ok(table)
}

#[extend::ext(name = VirtualAddressMapping)]
pub impl VirtualAddress {
    #[cfg(target_arch = "x86_64")]
    fn walk_map(&self) -> impl Iterator<Item = NonNull<PageTableEntry>> {
        let mut table = PageTable::current_table();
        let mut level = Some(PageTableLevel::PML4);
        let mut prev_entry: Option<NonNull<PageTableEntry>> = None;
        ::core::iter::from_fn(move || {
            if let Some(mut prev_entry) = prev_entry {
                table = match unsafe { prev_entry.as_ref().mapped_table() }
                {
                    Ok(t) => t,
                    Err(EntryError::NoMapping) => {
                        println!("ALLOCATED NEW TABLE");
                        alloc_table(unsafe { prev_entry.as_mut() })
                            .expect("Cannot allocate table")
                    }
                    Err(EntryError::NotATable) => {
                        todo!("Mapping to this entry already exists")
                    }
                };
            }

            let current_level = level?;
            let entry = unsafe {
                NonNull::from_ref(
                    &table.as_mut().entries[self.index_of(current_level)],
                )
            };
            level = current_level.next();
            prev_entry = Some(entry);

            Some(entry)
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
            .walk_map()
            .nth(page_size.mapping_table() as usize)
            .ok_or(MappingError::TableDoesNotExist)?;

        unsafe {
            entry.as_mut().map(
                address,
                flags.unwrap_or_else(|| match page_size {
                    PageSize::Regular => {
                        PageEntryFlags::regular_page_flags()
                    }
                    PageSize::Big | PageSize::Huge => {
                        PageEntryFlags::huge_page_flags()
                    }
                }),
            )
        }
        Ok(())
    }
}
