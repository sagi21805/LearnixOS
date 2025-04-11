use super::{
    address_types::{PhysicalAddress, VirtualAddress},
    get_current_page_table,
};
use crate::flag;
use constants::{enums::PageSize, values::PAGE_DIRECTORY_ENTRIES};
#[derive(Debug)]
pub struct PageTableEntry(u64);

#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    pub(crate) entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
}

impl PageTableEntry {
    flag!(present, 0);
    flag!(writable, 1);
    flag!(usr_access, 2);
    flag!(write_through_cache, 3);
    flag!(disable_cache, 4);

    // This flag can help identifying if an entry is the last one, or it is pointing to another directory
    flag!(huge_page, 7);
    flag!(global, 8);
    flag!(not_executable, 63);

    #[inline]
    // Writable by default
    pub(crate) const fn set_frame_address(&mut self, frame_base: PhysicalAddress) {
        if !self.present() {
            self.0 |= frame_base.as_usize() as u64 & 0xfffffffff_000;
            self.set_present();
            self.set_writable();
        } else {
            panic!("Page is already mapped");
        }
    }

    #[inline]
    pub(crate) const fn get_address(&self) -> PhysicalAddress {
        if self.present() {
            PhysicalAddress::new((self.0 & 0x0000_fffffffff_000) as usize)
        } else {
            panic!("Page does not mapped to any addrees");
        }
    }
    #[inline]
    pub(crate) const fn empty() -> Self {
        Self(0)
    }
    #[inline]
    pub(crate) const fn get_next_table_mut(&self) -> &mut PageTable {
        if !self.huge_page() {
            unsafe { core::mem::transmute::<PhysicalAddress, &mut PageTable>(self.get_address()) }
        } else {
            panic!("The page is authoritative so there is no next table");
        }
    }
    #[inline]
    pub(crate) const fn set_next_table(&mut self, page_table: &'static PageTable) {
        unsafe {
            self.set_frame_address(core::mem::transmute::<&'static PageTable, PhysicalAddress>(
                page_table,
            ));
        }
    }

    #[inline]
    // create table, set it using set_next_table and return a reference to it
    pub(crate) fn create_new_table(&self) -> &PageTable {
        let global_table = unsafe { get_current_page_table() };
        let (p, v) = global_table.find_avilable_page(PageSize::Regular);
        todo!()
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl PageTable {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            entries: [const { PageTableEntry::empty() }; PAGE_DIRECTORY_ENTRIES],
        }
    }

    #[inline]
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self as *const Self as usize)
    }

    #[inline]
    pub fn find_avilable_page(&self, page_size: PageSize) -> (PhysicalAddress, VirtualAddress) {
        todo!()
    }
}
