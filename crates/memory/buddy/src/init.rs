//! Building the free lists from a fresh arena.
//!
//! The steps are private, so only `build_free_lists` sets their order.

use core::ops::Range;

use common::{
    address_types::{Address, PhysicalAddress},
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    enums::{BuddyOrder, MemoryRegionType},
};
use x86::memory_map::MemoryMap;

use crate::{
    BuddyAllocatorInner,
    meta::{BuddyArena, BuddyBlock},
};

impl<Arena, Block> BuddyAllocatorInner<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    /// Reserves, links and merges, so the free lists hold every free
    /// page.
    pub(crate) fn initialize(&mut self, mmap: &MemoryMap) {
        self.reserve_unavailable(mmap);
        self.link_free_blocks();
        self.merge_free_blocks();
    }

    /// Reserves the regions that aren't usable and the memory that holds
    /// the arena itself.
    fn reserve_unavailable(&mut self, mmap: &MemoryMap) {
        self.reserve_unusable_regions(mmap);
        self.reserve(self.arena.storage());
    }

    /// Reserves every region that isn't usable, up to the last usable
    /// one, where the arena ends.
    fn reserve_unusable_regions(&mut self, mmap: &MemoryMap) {
        let regions = mmap.regions.read();
        let Some(last_usable) = regions
            .iter()
            .rposition(|r| r.region_type == MemoryRegionType::Usable)
        else {
            return;
        };

        for region in regions[..last_usable]
            .iter()
            .filter(|r| r.region_type != MemoryRegionType::Usable)
        {
            let start = region.base_address as usize;
            let end = start + region.length as usize;
            self.reserve(
                PhysicalAddress::from(start)..PhysicalAddress::from(end),
            );
        }
    }

    /// Marks every page that `range` touches allocated.
    ///
    /// It only sets flags and leaves the free lists alone, so it must run
    /// before they're built.
    fn reserve(&mut self, range: Range<PhysicalAddress>) {
        let start =
            range.start.align_down(REGULAR_PAGE_ALIGNMENT).as_usize();
        let end = range.end.align_up(REGULAR_PAGE_ALIGNMENT).as_usize();

        for address in (start..end).step_by(REGULAR_PAGE_SIZE) {
            let mut block = self
                .arena
                .page_with_address(PhysicalAddress::from(address))
                .expect("reserved memory lies inside the arena");
            // SAFETY: `page_with_address` returns a live block of the
            // arena.
            unsafe { block.as_mut() }
                .meta_mut()
                .flags
                .set_allocated(true);
        }
    }

    /// Links every free block into the order-0 free list, from the last
    /// block down, so the list starts at the lowest address.
    fn link_free_blocks(&mut self) {
        let len = self.arena.iter().len();
        for n in (0..len).rev() {
            let block = self.arena.at(n).expect("`n` is inside the arena");
            // SAFETY: `at` returns a live block of the arena.
            if !unsafe { block.as_ref() }.is_allocated() {
                self.attach(block);
            }
        }
    }

    /// Merges from every free block that heads its range, from the last
    /// block down, so each free list starts at its lowest address.
    fn merge_free_blocks(&mut self) {
        let len = self.arena.iter().len();
        for n in (0..len).rev() {
            let block = self.arena.at(n).expect("`n` is inside the arena");
            // SAFETY: `at` returns a live block of the arena.
            let page = unsafe { block.as_ref() };
            if !page.is_allocated() && page.order() != BuddyOrder::None {
                self.merge(block);
            }
        }
    }
}
