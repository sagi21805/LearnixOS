use core::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use common::{
    address_types::VirtualAddress,
    constants::{PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_SIZE},
    late_init::LateInit,
};

use crate::{
    memory::{
        memory_map::ParsedMemoryMap,
        page::{
            PAGES, UnassignedPage,
            meta::{BuddyPageMeta, PageMeta},
        },
    },
    println,
};

pub struct PageMap(NonNull<[UnassignedPage]>);

impl Deref for PageMap {
    type Target = [UnassignedPage];

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for PageMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl PageMap {
    /// Initializes all pages on the constant address
    /// ([`PAGE_ALLOCATOR_OFFSET`]) and returns the end address.
    pub fn init(
        uninit: &'static mut LateInit<PageMap>,
        mmap: ParsedMemoryMap,
    ) -> VirtualAddress {
        let last = mmap.last().unwrap();
        let last_address = (last.base_address + last.length) as usize;
        let total_pages = last_address / REGULAR_PAGE_SIZE;

        println!(
            "Last address: {}, Total Pages: {}, size_of_array: {:x?} Kib",
            last_address,
            total_pages,
            total_pages * size_of::<UnassignedPage>() / 1024
        );
        unsafe {
            let page_map = NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(
                    PAGE_ALLOCATOR_OFFSET as *mut UnassignedPage,
                ),
                total_pages,
            );

            uninit.write(PageMap(page_map));

            for p in uninit.as_mut().iter_mut() {
                core::ptr::write_volatile(
                    p as *mut UnassignedPage,
                    UnassignedPage::new(PageMeta {
                        buddy: ManuallyDrop::new(BuddyPageMeta::default()),
                    }),
                )
            }
            (PAGES.as_ptr_range().end as usize).into()
        }
    }
}
