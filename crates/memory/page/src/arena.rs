use core::{
    mem::{MaybeUninit, offset_of},
    ops::Range,
    ptr::NonNull,
};

use common::{
    address_types::{Address, PhysicalAddress},
    constants::REGULAR_PAGE_SIZE,
    enums::{BuddyOrder, MemoryRegionType},
    page_index::PageIndex,
};

use libk::println;
use x86::memory_map::MemoryMap;

use buddy::{
    OwnedBlock,
    meta::{BlockFlagBits, BuddyArena, BuddyBlock, BuddyError},
};
use slab::{
    SlabArena, SlabFlags,
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
        assert!(
            u32::try_from(total_pages - 1)
                .ok()
                .and_then(PageIndex::new)
                .is_some(),
            "every page has a `PageIndex`"
        );

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

    /// Returns the page at the given index, if one exists.
    fn at(&self, n: usize) -> Option<NonNull<Page>> {
        if n >= self.inner.len() {
            return None;
        }
        Some(unsafe { NonNull::from_ref(self.inner.get_unchecked(n)) })
    }

    fn index_of(&self, page: NonNull<Page>) -> PageIndex {
        debug_assert!(
            self.inner
                .as_ptr_range()
                .contains(&page.as_ptr().cast_const()),
            "the page is in this arena"
        );
        // SAFETY: `page` is a page of this arena.
        let offset = unsafe {
            page.as_ptr().offset_from_unsigned(self.inner.as_ptr())
        };
        PageIndex::new(offset as u32)
            .expect("`new` gives every page an index")
    }

    unsafe fn section_index_of(
        &self,
        page: NonNull<Page>,
    ) -> (usize, usize) {
        let offset = BuddyArena::index_of(self, page).get() as usize;
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
        let page = self.page_of(descriptor);
        SlabObjects::new(self.address_of(page).translate().as_non_null())
    }

    fn index_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> PageIndex {
        BuddyArena::index_of(self, self.page_of(descriptor))
    }

    /// The owner byte of the descriptor's page.
    fn slab_flags_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> SlabFlags {
        // SAFETY: `page_of` returns a live page of this arena.
        let page = unsafe { self.page_of(descriptor).as_ref() };
        SlabFlags::from(page.flags().owner())
    }

    fn set_slab_flags_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &mut SlabDescriptor<T, S>,
        flags: SlabFlags,
    ) {
        // SAFETY: only the slab that owns a page holds its descriptor by
        // `&mut`, and the slab gave the page's token up into its lists.
        let mut block =
            unsafe { OwnedBlock::from_raw(self.page_of(descriptor)) };
        block.set_owner(u8::from(flags));
        let _ = block.into_raw();
    }

    unsafe fn descriptor_at<T: Slab, S: SlabState>(
        &self,
        index: PageIndex,
    ) -> NonNull<SlabDescriptor<T, S>> {
        let page = self
            .at(index.get() as usize)
            .expect("a slab's index is inside the arena");
        // SAFETY: `at` returns a live page of this arena.
        NonNull::from_ref(&unsafe { page.as_ref() }.body).cast()
    }
}

impl PageMap {
    /// The page whose body holds `descriptor`.
    fn page_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> NonNull<Page> {
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
        page
    }

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
        let offset = BuddyArena::index_of(self, page).get() as usize;
        let section_offset = offset % (1 << BuddyOrder::MAX as usize);
        let section_idx = offset / (1 << BuddyOrder::MAX as usize);

        (section_idx, section_offset)
    }
}
