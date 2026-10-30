//! Where the page arena lives in physical memory, and the page tables
//! that map it.

use core::ops::Range;

use common::{
    address_types::{Address, PhysicalAddress},
    constants::{BIG_PAGE_SIZE, HUGE_PAGE_SIZE, MiB, REGULAR_PAGE_SIZE},
    enums::MemoryRegionType,
};
use x86::{
    memory_map::{MemoryMap, MemoryRegion},
    structures::paging::{PageEntryFlags, PageTable},
};

/// The memory that holds the arena. It starts with one L2 table for each
/// GiB past the first that the pages reach, because the boot map covers
/// only the first GiB, and the pages follow the tables.
pub struct ArenaPlacement {
    tables: Range<usize>,
    pages: Range<usize>,
}

impl ArenaPlacement {
    /// The first place in a usable region, at or above 1 MiB, that holds
    /// `size` bytes of pages and the tables that map them.
    pub fn find(mmap: &MemoryMap, size: usize) -> ArenaPlacement {
        mmap.regions
            .read()
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .find_map(|r| ArenaPlacement::inside(r, size))
            .expect("a usable region holds the page arena")
    }

    /// The placement at the start of `region`'s part at or above 1 MiB,
    /// if it fits there.
    ///
    /// The tables can only be written through the boot map, so a
    /// placement that needs tables must start in the first GiB.
    fn inside(region: &MemoryRegion, size: usize) -> Option<Self> {
        let start = (region.base_address as usize)
            .max(MiB)
            .next_multiple_of(REGULAR_PAGE_SIZE);
        let end = (region.base_address + region.length) as usize;

        let pages_after = |tables: usize| {
            let pages = start + tables * REGULAR_PAGE_SIZE;
            pages..pages + size
        };
        let tables = (0..)
            .find(|&n| gibs_past_first(pages_after(n)).len() <= n)
            .expect("one table per GiB is always enough");

        let placement = ArenaPlacement {
            tables: start..start + tables * REGULAR_PAGE_SIZE,
            pages: pages_after(tables),
        };
        (placement.pages.end <= end
            && (tables == 0 || start < HUGE_PAGE_SIZE))
            .then_some(placement)
    }

    /// The physical address of the first page.
    pub fn pages(&self) -> PhysicalAddress {
        PhysicalAddress::from(self.pages.start)
    }

    /// The memory the tables and the pages take.
    pub fn storage(&self) -> Range<PhysicalAddress> {
        PhysicalAddress::from(self.tables.start)
            ..PhysicalAddress::from(self.pages.end)
    }

    /// Identity-maps the pages past the first GiB with 2 MiB pages,
    /// through one carved L2 table per GiB.
    ///
    /// # Safety
    ///
    /// The placement's memory must be usable and unused, and the identity
    /// L3 table must map nothing past the first GiB.
    pub unsafe fn map(&self) {
        let pml4 = PageTable::current_table();
        // SAFETY: the boot map's PML4 maps the identity L3 table in its
        // first entry.
        let mut l3 = unsafe { pml4.as_ref() }.entries[0]
            .mapped_table()
            .expect("the boot map has an identity L3 table");

        let tables = self.tables.clone().step_by(REGULAR_PAGE_SIZE);
        for (gib, table) in gibs_past_first(self.pages.clone()).zip(tables)
        {
            let mut l2 =
                PhysicalAddress::from(table).as_non_null::<PageTable>();
            // SAFETY: `table` is unused memory in the first GiB, which
            // the boot map identity-maps.
            let l2 = unsafe {
                l2.write(PageTable::empty());
                l2.as_mut()
            };
            self.fill(l2, gib);

            // SAFETY: nothing maps this GiB yet, and `l2` maps only the
            // arena's pages in it.
            unsafe {
                l3.as_mut()
                    .entries
                    .get_mut(gib)
                    .expect("the page arena ends below 512 GiB")
                    .map_unchecked(
                        PhysicalAddress::from(table),
                        PageEntryFlags::table_flags(),
                    )
            };
        }
    }

    /// Points the entries of `l2` that cover the pages inside `gib` at
    /// the same physical memory.
    fn fill(&self, l2: &mut PageTable, gib: usize) {
        let gib_start = gib * HUGE_PAGE_SIZE;
        let first = self.pages.start.max(gib_start) - gib_start;
        let last = (self.pages.end - 1)
            .min(gib_start + HUGE_PAGE_SIZE - 1)
            - gib_start;

        for i in first / BIG_PAGE_SIZE..=last / BIG_PAGE_SIZE {
            // SAFETY: `l2` is fresh, and `gib_start + i * 2 MiB` is 2 MiB
            // aligned.
            unsafe {
                l2.entries[i].map_unchecked(
                    PhysicalAddress::from(gib_start + i * BIG_PAGE_SIZE),
                    PageEntryFlags::huge_page_flags(),
                )
            };
        }
    }
}

/// The GiBs past the first that `range` touches.
fn gibs_past_first(range: Range<usize>) -> Range<usize> {
    let first = (range.start / HUGE_PAGE_SIZE).max(1);
    let last = (range.end - 1) / HUGE_PAGE_SIZE;
    first..(last + 1).max(first)
}
