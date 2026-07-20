extern crate alloc;

use core::ptr::NonNull;

use common::{
    address_types::{Address, PhysicalAddress},
    constants::REGULAR_PAGE_SIZE,
    enums::{BuddyOrder, MemoryRegionType},
};

use alloc::boxed::Box;

use libk::println;
use x86::memory_map::MemoryMap;

use buddy::meta::{
    BuddyArena, BuddyBlock, BuddyError, BuddyFlags, BuddyMeta, Head,
    Regular,
};

use crate::{Page, meta::PageMeta};

pub struct PageMap {
    inner: Box<[Page]>,
}

impl BuddyArena<Page> for PageMap {
    fn new(mmap: &MemoryMap, head: &mut BuddyMeta<Head>) -> Self {
        let regions = mmap.regions.read();

        let last = regions
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .last()
            .expect("Memory map is empty, no useble region found");

        let last_address = (last.base_address + last.length) as usize;
        let total_pages = last_address / REGULAR_PAGE_SIZE;
        println!("Total pages: {}", total_pages);
        unsafe {
            let mut page_map =
                Box::new_uninit_slice(total_pages).assume_init();
            println!("Last Memory Address: {:?}", page_map.as_ptr_range());

            let mut prev = &mut page_map[0];

            *prev = Page {
                meta: PageMeta {
                    buddy: BuddyMeta::<Regular>::new(
                        NonNull::from_ref(head),
                        BuddyFlags::new()
                            .order(BuddyOrder::Order0)
                            .allocated(false),
                    ),
                },
            };

            head.attach_block(NonNull::from_ref(prev));

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

    fn page_with_address(
        &self,
        address: PhysicalAddress,
    ) -> Result<NonNull<Page>, BuddyError> {
        let offset = address.as_usize() / REGULAR_PAGE_SIZE;
        Ok(NonNull::from_ref(
            self.inner.get(offset).ok_or(BuddyError::PageNotInArena)?,
        ))
    }

    /// Because this is an array, the
    fn buddy_of(
        &self,
        block: NonNull<Page>,
    ) -> Result<NonNull<Page>, BuddyError> {
        let order = unsafe { block.as_ref().meta.buddy.flags.get_order() };

        let (section_idx, section_offset) =
            unsafe { self.section_index_of(block) };
        let buddy_idx = match order {
            BuddyOrder::None => return Err(BuddyError::PageInLargerOrder),
            BuddyOrder::MAX => return Err(BuddyError::MaxOrder),
            _ => {
                (section_offset ^ (1 << order as usize))
                    + section_idx * (1 << BuddyOrder::MAX as usize)
            }
        };

        Ok(self.at(buddy_idx).ok_or(BuddyError::BuddyOutOfRange)?)
    }

    #[inline]
    fn iter(&self) -> impl ExactSizeIterator<Item = NonNull<Page>> {
        self.inner.as_ref().iter().map(NonNull::from_ref)
    }

    fn merge(
        &self,
        mut block: NonNull<Page>,
        mut buddy: NonNull<Page>,
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

        let detached_block = unsafe { block.as_mut().meta_mut().detach() };
        let detached_buddy = unsafe { buddy.as_mut().meta_mut().detach() };

        let (mut l, mut r) = if detached_block < detached_buddy {
            (detached_block, detached_buddy)
        } else {
            (detached_buddy, detached_block)
        };

        unsafe {
            l.as_mut().flags.set_order(next_order);

            r.as_mut().flags.set_order(BuddyOrder::None);
        }

        Ok(Page::from_meta(l))
    }

    fn split(
        &self,
        mut block: NonNull<Page>,
    ) -> Result<(NonNull<Page>, NonNull<Page>), BuddyError> {
        let mut detached = unsafe { block.as_mut().meta_mut().detach() };

        let prev_order = unsafe {
            detached
                .as_ref()
                .flags
                .get_order()
                .prev()
                .ok_or(BuddyError::Unsplitable)?
        };

        // First set the order of the current block to find the current
        // buddy
        unsafe {
            detached.as_mut().flags.set_order(prev_order);
        }

        let mut buddy = self.buddy_of(block)?;

        unsafe {
            buddy.as_mut().meta.buddy.flags.set_order(prev_order);
        }

        Ok((block, buddy))
    }

    /// Returns the page at the given index, if one exists.
    fn at(&self, n: usize) -> Option<NonNull<Page>> {
        if n >= self.inner.len() {
            return None;
        }
        Some(unsafe { NonNull::from_ref(self.inner.get_unchecked(n)) })
    }

    unsafe fn section_index_of(
        &self,
        page: NonNull<Page>,
    ) -> (usize, usize) {
        let offset = unsafe { self.index_of(page) };
        let section_offset = offset % (1 << BuddyOrder::MAX as usize);
        let section_idx = offset / (1 << BuddyOrder::MAX as usize);

        (section_idx, section_offset)
    }
}

impl PageMap {
    /// Returns the index of the given page in the arena.
    ///
    /// # Safety
    ///
    /// The page must be contained within the arena's memory range.
    pub unsafe fn index_of(&self, page: NonNull<Page>) -> usize {
        debug_assert!(
            self.inner
                .as_ptr_range()
                .contains(&(page.as_ptr() as *const _))
        );
        unsafe { page.as_ptr().offset_from_unsigned(self.inner.as_ptr()) }
    }
}

impl PageMap {
    /// Returns the (section index, section offset) of the given page.
    ///
    /// A section is a contiguous range of pages aligned to
    /// [`BuddyOrder::MAX`].
    ///
    /// # Safety
    ///
    /// The page must be contained within the arena's memory range.
    pub unsafe fn section_index_of(
        &self,
        page: NonNull<Page>,
    ) -> (usize, usize) {
        let offset = unsafe { self.index_of(page) };
        let section_offset = offset % (1 << BuddyOrder::MAX as usize);
        let section_idx = offset / (1 << BuddyOrder::MAX as usize);

        (section_idx, section_offset)
    }
}
