use core::{
    default,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use common::{
    address_types::VirtualAddress,
    constants::{PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_SIZE},
    enums::BUDDY_MAX_ORDER,
    late_init::LateInit,
};

use x86::memory_map::MemoryMap;

use buddy::{
    BuddyAllocator,
    meta::{BuddyMeta, Dummy, Real},
};

use crate::Page;

pub struct PageMap(NonNull<[Page]>);

impl Deref for PageMap {
    type Target = [Page];

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for PageMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
pub fn init(&'static mut self, arena: NonNull<Arena>) {
    for block in unsafe { arena.as_ref().iter() } {
        let order =
            match unsafe { block.as_ref().meta().flags.get_order() } {
                BuddyOrder::None => continue,
                o => o as usize,
            };

        self.freelist[order]
            .attach(NonNull::from_ref(unsafe { block.as_ref().meta() }));
    }
}
impl PageMap {
    /// Initializes all pages on the constant address
    /// ([`PAGE_ALLOCATOR_OFFSET`]) and returns the end address.
    pub fn init(uninit: &'static mut LateInit<PageMap>, mmap: MemoryMap) {
        let last = mmap.last().unwrap();
        let last_address = (last.base_address + last.length) as usize;
        let total_pages = last_address / REGULAR_PAGE_SIZE;

        let freelist = [BuddyMeta::<Dummy>::default(); BUDDY_MAX_ORDER];

        unsafe {
            let page_map = NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(PAGE_ALLOCATOR_OFFSET as *mut Page),
                total_pages,
            );

            let init = uninit.write(PageMap(page_map));

            for p in init.iter_mut() {
                core::ptr::write_volatile(
                    p as *mut Page,
                    Page {
                        meta: PageMeta {
                            buddy: ManuallyDrop::new(
                                BuddyMeta::<Real>::default(),
                            ),
                        },
                    },
                )
            }
        }
    }
}
