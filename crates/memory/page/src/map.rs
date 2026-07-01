extern crate alloc;

use core::ptr::NonNull;

use common::{
    address_types::{Address, PhysicalAddress},
    constants::REGULAR_PAGE_SIZE,
    enums::{BuddyOrder, MemoryRegionType},
};

use alloc::boxed::Box;

use x86::memory_map::MemoryMap;

use buddy::meta::{
    BuddyArena, BuddyBlock, BuddyError, BuddyFlags, BuddyMeta, Head,
    Regular,
};

use crate::{Page, meta::PageMeta};

pub struct PageMap {
    inner: Box<[Page]>,
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
    fn new(mmap: &MemoryMap, heads: &[BuddyMeta<Head>]) -> Self {
        let regions = mmap.regions.lock();

        let last = regions
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .last()
            .expect("Memory map is empty, no useble region found");
        let last_address = (last.base_address + last.length) as usize;
        let total_pages = last_address / REGULAR_PAGE_SIZE;

        unsafe {
            let mut page_map =
                Box::new_uninit_slice(total_pages).assume_init();

            let mut prev = &mut page_map[0];

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

            for i in 0..page_map.len().saturating_sub(1) {
                let (left, right) = page_map.split_at_mut(i + 1);
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

            PageMap { inner: page_map }
        }
    }

    fn address_of(&self, block: NonNull<Page>) -> PhysicalAddress {
        unsafe {
            let offset =
                block.as_ptr().offset_from_unsigned(self.inner.as_ptr());

            PhysicalAddress::new_unchecked(offset * REGULAR_PAGE_SIZE)
        }
    }

    /// Because this is an array, the
    fn buddy_of(
        &self,
        block: NonNull<Page>,
    ) -> Result<NonNull<Page>, BuddyError> {
        let order = unsafe { block.as_ref().meta.buddy.flags.get_order() };

        let offset = unsafe {
            block.as_ptr().offset_from_unsigned(self.inner.as_ptr())
                / size_of::<Page>()
        };
        let section_offset = offset % (1 << BuddyOrder::MAX as usize);
        let section_idx = offset / (1 << BuddyOrder::MAX as usize);

        let buddy_idx = match order {
            BuddyOrder::None => return Err(BuddyError::PageInLargerOrder),
            BuddyOrder::MAX => return Err(BuddyError::MaxOrder),
            _ => {
                (section_offset ^ (1 << order as usize))
                    + section_idx * (1 << BuddyOrder::MAX as usize)
            }
        };

        Ok(NonNull::from_ref(&self.inner[buddy_idx]))
    }

    #[inline]
    fn iter(&self) -> impl ExactSizeIterator<Item = NonNull<Page>> {
        self.inner.as_ref().iter().map(NonNull::from_ref)
    }

    fn merge(
        &self,
        block: NonNull<Page>,
        buddy: NonNull<Page>,
    ) -> Result<NonNull<Page>, BuddyError> {
        debug_assert_eq!(self.buddy_of(block)?, buddy);
        debug_assert!(unsafe {
            block.as_ref().meta.buddy.flags.get_order()
                == buddy.as_ref().meta.buddy.flags.get_order()
        });
        debug_assert!(
            unsafe { block.as_ref().meta.buddy.flags.get_order() }
                != BuddyOrder::None
        );
        debug_assert!(unsafe {
            !block.as_ref().meta.buddy.flags.is_allocated()
        });
        debug_assert!(unsafe {
            !buddy.as_ref().meta.buddy.flags.is_allocated()
        });

        let next_order = unsafe {
            block
                .as_ref()
                .meta
                .buddy
                .flags
                .get_order()
                .next()
                .ok_or(BuddyError::MaxOrder)?
        };

        let mut detached_block = self.detach_mid(block);
        let mut detached_buddy = self.detach_mid(buddy);
        unsafe {
            detached_block
                .as_mut()
                .meta
                .buddy
                .flags
                .set_order(next_order);

            detached_buddy
                .as_mut()
                .meta
                .buddy
                .flags
                .set_order(next_order);
        }

        Ok(detached_block.min(detached_buddy))
    }

    fn split(
        &self,
        block: NonNull<Page>,
    ) -> Result<(NonNull<Page>, NonNull<Page>), BuddyError> {
        todo!()
    }

    fn detach_mid(&self, block: NonNull<Page>) -> NonNull<Page> { todo!() }
}
