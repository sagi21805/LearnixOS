#[macro_use]
pub mod entry_flags;
pub mod init;
pub mod page_table;
pub mod page_table_entry;

use core::ptr::NonNull;

use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    enums::PageTableLevel,
};
pub use entry_flags::*;
#[cfg(target_arch = "x86")]
pub use init::*;
pub use page_table::*;
pub use page_table_entry::*;

#[cfg(target_arch = "x86_64")]
#[extend::ext]
pub impl VirtualAddress {
    fn walk(
        &self,
    ) -> impl Iterator<Item = (PageTableLevel, NonNull<PageTableEntry>)>
    {
        let mut table = Some(PageTable::current_table());
        let mut level = Some(PageTableLevel::PML4);
        ::core::iter::from_fn(move || {
            let current_level = level?;

            let entry = unsafe {
                &table?.as_ref().entries[self.index_of(current_level)]
            };

            if entry.get_flags().is_present() {
                table = entry.mapped_table().ok();
                level = current_level.next();
            } else {
                // Stop at the next iteration.
                level = None;
            }
            Some((current_level, NonNull::from_ref(entry)))
        })
    }

    fn translate(&self) -> Option<PhysicalAddress> {
        let (level, last) = self.walk().last()?;
        let last = unsafe { last.as_ref() };
        let flags = last.get_flags();

        let translatable = match level {
            PageTableLevel::PML4 => return None,
            PageTableLevel::PDPT | PageTableLevel::PD => {
                flags.is_present() && flags.is_huge_page()
            }
            PageTableLevel::PT => flags.is_present(),
        };

        translatable.then(|| unsafe {
            PhysicalAddress::new_unchecked(
                last.get_address().as_usize() + self.address_offset(level),
            )
        })
    }

    fn is_mapped(&self) -> bool {
        self.walk()
            .all(|(_, e)| unsafe { e.as_ref().get_flags().is_present() })
    }

    fn address_offset(&self, level: PageTableLevel) -> usize {
        // For a regualar page the offset is 12 bits, and then 9 is added
        // for each level.
        let mask = usize::MAX
            >> (usize::BITS as usize - (12 + 9 * level.level_number()));

        self.as_usize() & mask
    }
}
