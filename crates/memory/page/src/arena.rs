use core::{
    mem::{MaybeUninit, offset_of},
    ops::Range,
    ptr::NonNull,
};

use common::{
    address_types::{Address, PhysicalAddress},
    constants::REGULAR_PAGE_SIZE,
    enums::{BuddyOrder, MemoryRegionType},
};

use libk::println;
use x86::memory_map::MemoryMap;

use buddy::meta::{BlockFlagBits, BuddyArena, BuddyBlock, BuddyError};
use slab::{
    SlabArena,
    descriptor::{SlabDescriptor, SlabState},
    slab_objects::SlabObjects,
    traits::Slab,
};

use crate::{Page, placement::ArenaPlacement};

pub struct PageMap {
    inner: &'static [Page],
    placement: ArenaPlacement,
}

impl BuddyArena<Page> for PageMap {
    fn new(mmap: &MemoryMap) -> Self {
        let regions = mmap.regions.read();

        let last = regions
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .last()
            .expect("Memory map is empty, no useble region found");

        let last_address = (last.base_address + last.length) as usize;
        let total_pages = last_address / REGULAR_PAGE_SIZE;
        println!("Total pages: {}", total_pages);

        // SAFETY: nothing uses the usable memory yet, and the boot map
        // maps nothing past the first GiB.
        let placement = unsafe {
            ArenaPlacement::find(mmap, total_pages * size_of::<Page>())
        };

        // SAFETY: the placement holds `total_pages` pages, mapped and
        // unused.
        let page_map = unsafe {
            core::slice::from_raw_parts_mut(
                placement
                    .pages()
                    .as_non_null::<MaybeUninit<Page>>()
                    .as_ptr(),
                total_pages,
            )
        };
        println!("Last Memory Address: {:?}", page_map.as_ptr_range());
        for page in page_map.iter_mut() {
            page.write(Page::new(
                BlockFlagBits::new().order(BuddyOrder::Order0),
            ));
        }

        let page_map: &'static [MaybeUninit<Page>] = page_map;
        PageMap {
            // SAFETY: the loop wrote every page.
            inner: unsafe { page_map.assume_init_ref() },
            placement,
        }
    }

    fn storage(&self) -> Range<PhysicalAddress> {
        self.placement.storage()
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
        let order = unsafe { block.as_ref() }.flags().order();

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
        self.inner.iter().map(NonNull::from_ref)
    }

    fn merge(
        &self,
        block: NonNull<Page>,
        buddy: NonNull<Page>,
    ) -> Result<NonNull<Page>, BuddyError> {
        debug_assert_eq!(self.buddy_of(block)?, buddy);
        // SAFETY: `block` and `buddy` are live pages of this arena.
        let (block_page, buddy_page) =
            unsafe { (block.as_ref(), buddy.as_ref()) };
        let order = block_page.flags().order();
        debug_assert!(order == buddy_page.flags().order());
        debug_assert!(order != BuddyOrder::None);
        debug_assert!(!block_page.flags().is_allocated());
        debug_assert!(!buddy_page.flags().is_allocated());
        debug_assert!(
            // SAFETY: both pages are free, so buddy owns them.
            unsafe {
                !block_page.links().as_ref().is_linked()
                    && !buddy_page.links().as_ref().is_linked()
            },
            "the blocks to merge are in no free list"
        );

        let next_order = order.next().ok_or(BuddyError::MaxOrder)?;
        let (l, r) = if block < buddy {
            (block_page, buddy_page)
        } else {
            (buddy_page, block_page)
        };
        l.flags().modify(|bits| bits.set_order(next_order));
        r.flags().modify(|bits| bits.set_order(BuddyOrder::None));

        Ok(NonNull::from_ref(l))
    }

    fn split(
        &self,
        block: NonNull<Page>,
    ) -> Result<(NonNull<Page>, NonNull<Page>), BuddyError> {
        // SAFETY: `block` is a live page of this arena.
        let page = unsafe { block.as_ref() };
        debug_assert!(
            // SAFETY: a block to split is free, so buddy owns it.
            !unsafe { page.links().as_ref() }.is_linked(),
            "the block to split is in no free list"
        );

        let prev_order =
            page.flags().order().prev().ok_or(BuddyError::Unsplitable)?;

        // The block's new order finds its buddy at that order.
        page.flags().modify(|bits| bits.set_order(prev_order));
        let buddy = self.buddy_of(block)?;
        // SAFETY: `buddy_of` returns a live page of this arena.
        unsafe { buddy.as_ref() }
            .flags()
            .modify(|bits| bits.set_order(prev_order));

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

impl SlabArena for PageMap {
    /// The slots at the address buddy handed out for the descriptor's
    /// page.
    fn objects_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> SlabObjects<T> {
        // SAFETY: a descriptor lives in the body of its page.
        let page = unsafe {
            NonNull::from_ref(descriptor).byte_sub(offset_of!(Page, body))
        }
        .cast::<Page>();
        debug_assert!(
            self.inner
                .as_ptr_range()
                .contains(&page.as_ptr().cast_const()),
            "the descriptor is in a page of this arena"
        );

        SlabObjects::new(self.address_of(page).translate().as_non_null())
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
