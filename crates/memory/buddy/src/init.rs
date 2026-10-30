//! Building the free lists from a fresh arena.
//!
//! Each step returns a proof that it ran, and the next step takes it.
//! Only this module can make a proof, so the steps run in one order, and
//! reserving after the free lists are built doesn't compile.

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

/// Proof that the free lists aren't built yet.
struct Fresh(());

/// Proof that every page the allocator must not hand out is reserved.
struct Reserved(());

/// Proof that every free page is in the order-0 free list.
struct Linked(());

impl<Arena, Block> BuddyAllocatorInner<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    /// Reserves, links and merges, so the free lists hold every free
    /// page.
    pub(crate) fn build_free_lists(&mut self, mmap: &MemoryMap) {
        let reserved = self.reserve_unavailable(Fresh(()), mmap);
        let linked = self.link_free_blocks(reserved);
        self.merge_free_blocks(linked);
    }

    /// Reserves the regions that aren't usable and the memory that holds
    /// the arena itself.
    fn reserve_unavailable(
        &mut self,
        fresh: Fresh,
        mmap: &MemoryMap,
    ) -> Reserved {
        self.reserve_unusable_regions(mmap, &fresh);
        self.reserve(self.arena.storage(), &fresh);
        Reserved(())
    }

    /// Reserves every region that isn't usable, up to the last usable
    /// one, where the arena ends.
    fn reserve_unusable_regions(
        &mut self,
        mmap: &MemoryMap,
        fresh: &Fresh,
    ) {
        let regions = mmap.regions.read();
        let Some(last_usable) = regions
            .iter()
            .rposition(|r| r.region_type == MemoryRegionType::Usable)
        else {
            return;
        };

        for region in regions[..=last_usable]
            .iter()
            .filter(|r| r.region_type != MemoryRegionType::Usable)
        {
            let start = region.base_address as usize;
            let end = start + region.length as usize;
            self.reserve(
                PhysicalAddress::from(start)..PhysicalAddress::from(end),
                fresh,
            );
        }
    }

    /// Marks every page that `range` touches allocated.
    ///
    /// It only sets flags and leaves the free lists alone, so it needs
    /// them unbuilt.
    fn reserve(&mut self, range: Range<PhysicalAddress>, _: &Fresh) {
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
    fn link_free_blocks(&mut self, _: Reserved) -> Linked {
        let len = self.arena.iter().len();
        for n in (0..len).rev() {
            let block = self.arena.at(n).expect("`n` is inside the arena");
            // SAFETY: `at` returns a live block of the arena.
            if !unsafe { block.as_ref() }.is_allocated() {
                self.attach(block);
            }
        }
        Linked(())
    }

    /// Merges from every free block that heads its range, from the last
    /// block down, so each free list starts at its lowest address.
    fn merge_free_blocks(&mut self, _: Linked) {
        let len = self.arena.iter().len();
        for n in (0..len).rev() {
            let block = self.arena.at(n).expect("`n` is inside the arena");
            // SAFETY: `at` returns a live block of the arena.
            let heads_free_range = unsafe {
                !block.as_ref().is_allocated()
                    && block.as_ref().order() != BuddyOrder::None
            };
            if heads_free_range {
                self.merge(block);
            }
        }
    }
}
