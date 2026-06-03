use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use bump::BumpAllocator;
use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    enums::{PageSize, PageTableLevel},
    error::{EntryError, MappingError},
    late_init::LateInit,
};
use x86::structures::paging::{PageEntryFlags, PageTable, PageTableEntry};

pub static mut BUMP_ALLOCATOR: LateInit<BumpAllocator> =
    LateInit::uninit();

pub fn alloc_table<A: Allocator>(
    allocator: &A,
) -> Result<NonNull<PageTable>, AllocError> {
    allocator
        .allocate(Layout::new::<PageTable>())
        .map(|p| p.cast())
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
    fn walk_map(
        &self,
        allocator: &impl Allocator,
    ) -> impl Iterator<Item = NonNull<PageTableEntry>> {
        let mut table = PageTable::current_table();
        let mut level = Some(PageTableLevel::PML4);
        ::core::iter::from_fn(move || {
            let entry =
                unsafe { &table.as_ref().entries[self.index_of(level?)] };

            table = match entry.mapped_table() {
                Ok(t) => t,
                Err(EntryError::NoMapping) => {
                    alloc_table(allocator).expect("Cannot allocate table")
                }
                Err(EntryError::NotATable) => {
                    todo!("Mapping to this entry already exists")
                }
            };
            level = level?.next();
            Some(NonNull::from_ref(entry))
        })
    }

    #[cfg(target_arch = "x86_64")]
    fn map(
        &self,
        address: PhysicalAddress,
        flags: Option<PageEntryFlags>,
        page_size: PageSize,
        allocator: &impl Allocator,
    ) -> Result<(), MappingError> {
        let mut entry = self
            .walk_map(allocator)
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
