extern crate alloc;

use core::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    constants::{PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_SIZE},
    enums::{BUDDY_MAX_ORDER, BuddyOrder, MemoryRegionType},
    late_init::LateInit,
};

use alloc::boxed::Box;

use x86::memory_map::MemoryMap;

use buddy::meta::{
    BuddyArena, BuddyBlock, BuddyError, BuddyFlags, BuddyMeta,
    BuddyMetaType, Detached, Head, Regular,
};

use crate::{Page, meta::PageMeta};

pub struct PageMap(Box<[Page]>);

impl Deref for PageMap {
    type Target = [Page];

    fn deref(&self) -> &Self::Target { self.0.as_ref() }
}

impl DerefMut for PageMap {
    fn deref_mut(&mut self) -> &mut Self::Target { self.0.as_mut() }
}
// pub fn init(&'static mut self, arena: NonNull<Arena>) {
//     for block in unsafe { arena.as_ref().iter() } {
//         let order =
//             match unsafe { block.as_ref().meta().flags.get_order() } {
//                 BuddyOrder::None => continue,
//                 o => o as usize,
//             };

//         self.freelist[order]
//             .attach(NonNull::from_ref(unsafe { block.as_ref().meta()
// }));     }
// }

impl BuddyArena<Page> for PageMap {
    /// Initializes all pages on the constant address
    /// ([`PAGE_ALLOCATOR_OFFSET`]) and returns the end address.
    fn init(
        uninit: &'static mut LateInit<PageMap>,
        mmap: MemoryMap,
        heads: &[BuddyMeta<Head>],
    ) {
        let regions = mmap.regions.lock();

        let last = regions
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .last()
            .expect("Memory map is empty, no useble region found");
        let last_address = (last.base_address + last.length) as usize;
        let total_pages = last_address / REGULAR_PAGE_SIZE;

        unsafe {
            let page_map = Box::new_uninit_slice(total_pages);

            let init = uninit.init(PageMap(page_map.assume_init()));

            let mut prev = &mut init.0[0];

            *prev = Page {
                meta: PageMeta {
                    buddy: BuddyMeta::<Regular>::new(
                        NonNull::from_ref(&heads[0]),
                        BuddyFlags::new()
                            .order(BuddyOrder::Order0)
                            .allocated(false),
                    ),
                },
            };

            for i in 0..init.len().saturating_sub(1) {
                let (left, right) = init.split_at_mut(i + 1);
                prev = left.last_mut().unwrap();
                let next = right.first_mut().unwrap();
                *next = Page {
                    meta: PageMeta {
                        buddy: BuddyMeta::<Regular>::new(
                            NonNull::from_ref(prev.meta()),
                            BuddyFlags::new()
                                .order(BuddyOrder::Order0)
                                .allocated(false),
                        ),
                    },
                };
                prev.meta.buddy.attach_block(NonNull::from_mut(next));
            }
        }
    }

    fn address_of(&self, block: NonNull<Page>) -> PhysicalAddress {
        unsafe {
            let offset =
                block.as_ptr().offset_from_unsigned(self.as_ptr());

            PhysicalAddress::new_unchecked(offset * REGULAR_PAGE_SIZE)
        }
    }

    fn buddy_of(
        &self,
        block: NonNull<Page>,
    ) -> Result<NonNull<Page>, BuddyError> {
        todo!()
    }

    #[inline]
    fn iter(&self) -> impl ExactSizeIterator<Item = NonNull<Page>> {
        self.as_ref().iter().map(NonNull::from_ref)
    }

    fn merge(
        &self,
        block: NonNull<Page>,
        buddy: NonNull<Page>,
    ) -> Result<NonNull<Page>, BuddyError> {
        todo!()
    }

    fn split(
        &self,
        block: NonNull<Page>,
    ) -> Result<(NonNull<Page>, NonNull<Page>), BuddyError> {
        todo!()
    }
}

unsafe impl Sync for PageMap {}
