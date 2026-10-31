//! Where the page arena lives in physical memory, and the page tables
//! that map it.

use core::{iter::StepBy, ops::Range};

use common::{
    address_types::{Address, PhysicalAddress},
    constants::{
        BIG_PAGE_SIZE, BOOT_MAPPED_GIBS, HUGE_PAGE_SIZE, MiB,
        REGULAR_PAGE_SIZE,
    },
    enums::MemoryRegionType,
};
use x86::{
    memory_map::{MemoryMap, MemoryRegion},
    structures::paging::{PageEntryFlags, PageTable},
};

/// The memory that holds the arena. It starts with one L2 table for each
/// GiB past the first that the pages reach, because the boot map covers
/// only the first GiB, and the pages follow the tables.
///
/// The layout for the arena there for would be
///
/// |Table|Table|...|Page|Page|Page|...
///
/// *Note:* Extra tables may not be needed.
pub struct ArenaPlacement {
    tables: Range<usize>,
    pages: Range<usize>,
}

impl ArenaPlacement {
    /// Places `size` bytes of pages, and the tables that map them, at the
    /// start of the first usable region at or above 1 MiB, and maps them.
    ///
    /// # Safety
    ///
    /// Nothing may use the usable memory yet, and the identity L3 table
    /// must map nothing past the first GiB.
    pub unsafe fn find(mmap: &MemoryMap, size: usize) -> ArenaPlacement {
        let placement = mmap
            .regions
            .read()
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .find_map(|r| unsafe { ArenaPlacement::at_start_of(r, size) })
            .expect("a usable region holds the page arena");
        // SAFETY: the placement is in usable memory, which the caller
        // promises is unused.
        unsafe { placement.map() };
        placement
    }

    /// The tables and pages at the start of `region`'s part at or above
    /// 1 MiB, if `region` is usable and they fit in it.
    ///
    /// # Safety
    ///
    /// Region must be a useable memory region.
    unsafe fn at_start_of(
        region: &MemoryRegion,
        size: usize,
    ) -> Option<Self> {
        debug_assert!(region.region_type == MemoryRegionType::Usable);
        let table_start = (region.base_address as usize)
            .max(MiB)
            .next_multiple_of(REGULAR_PAGE_SIZE);
        let end = (region.base_address + region.length) as usize;

        let tables = tables_for(table_start, size);
        let pages_start = table_start + tables * size_of::<PageTable>();

        let fits = pages_start + size <= end;
        let tables_are_boot_mapped = tables == 0
            || pages_start <= BOOT_MAPPED_GIBS * HUGE_PAGE_SIZE;

        (fits && tables_are_boot_mapped).then(|| ArenaPlacement {
            tables: table_start..pages_start,
            pages: pages_start..pages_start + size,
        })
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

    /// Identity-maps the unmapped GiBs of the pages with 2 MiB pages,
    /// through one carved L2 table per GiB.
    ///
    /// # Safety
    ///
    /// The placement's memory must be usable and unused, and the identity
    /// L3 table must map nothing past the first GiB.
    unsafe fn map(&self) {
        let pml4 = PageTable::current_table();
        // SAFETY: the boot map's PML4 maps the identity L3 table in its
        // first entry.
        let mut l3 = unsafe { pml4.as_ref() }.entries[0]
            .mapped_table()
            .expect("the boot map has an identity L3 table");

        let tables = self.tables.clone().step_by(size_of::<PageTable>());
        for (gib, table) in unmapped_gibs(self.pages.clone()).zip(tables) {
            // SAFETY: `table` is unused memory in the first GiB, which
            // the boot map identity-maps.
            unsafe {
                PhysicalAddress::from(table)
                    .as_non_null::<PageTable>()
                    .write(self.gib_table(gib))
            };

            // SAFETY: nothing maps this GiB yet, and `table` maps only
            // the arena's pages in it.
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

    /// An L2 table that identity-maps the 2 MiB pages of GiB `gib` that
    /// the arena's pages touch.
    fn gib_table(&self, gib: usize) -> PageTable {
        let mut l2 = PageTable::empty();
        let touched =
            l2.entries.iter_mut().zip(big_frames_of(gib)).filter(
                |&(_, frame)| {
                    self.pages.start < frame + BIG_PAGE_SIZE
                        && self.pages.end > frame
                },
            );
        for (entry, frame) in touched {
            // SAFETY: `l2` is fresh, and `frame` is 2 MiB aligned.
            unsafe {
                entry.map_unchecked(
                    PhysicalAddress::from(frame),
                    PageEntryFlags::huge_page_flags(),
                )
            };
        }
        l2
    }
}

/// The fewest L2 tables that map `size` bytes of pages placed at `start`
/// right after those tables.
fn tables_for(table_start: usize, size: usize) -> usize {
    (0..)
        .find(|&tables| {
            let pages_start =
                table_start + tables * size_of::<PageTable>();
            unmapped_gibs(pages_start..pages_start + size).len() <= tables
        })
        .expect("one table per GiB is always enough")
}

/// The GiBs `pages` reaches that the boot map doesn't cover.
fn unmapped_gibs(pages: Range<usize>) -> Range<usize> {
    let first = (pages.start / HUGE_PAGE_SIZE).max(BOOT_MAPPED_GIBS);
    let last = (pages.end - 1) / HUGE_PAGE_SIZE;
    first..last + 1
}

/// The 2 MiB frames of GiB `gib`, in order.
fn big_frames_of(gib: usize) -> StepBy<Range<usize>> {
    let start = gib * HUGE_PAGE_SIZE;
    (start..start + HUGE_PAGE_SIZE).step_by(BIG_PAGE_SIZE)
}
