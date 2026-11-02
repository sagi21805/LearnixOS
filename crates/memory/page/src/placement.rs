//! Where the page arena lives in physical memory, and the page tables
//! that map all of RAM.

use core::{iter::StepBy, ops::Range, ptr::NonNull};

use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    constants::{
        BIG_PAGE_SIZE, BOOT_MAPPED_GIBS, HUGE_PAGE_SIZE, MiB,
        PHYSICAL_MEMORY_OFFSET, REGULAR_PAGE_SIZE,
    },
    enums::{MemoryRegionType, PageTableLevel},
};
use x86::{
    memory_map::{MemoryMap, MemoryRegion},
    structures::paging::{PageEntryFlags, PageTable},
};

/// The memory that holds the arena. It starts with one L2 table for each
/// GiB of RAM past the first, because the boot map covers only the first
/// GiB, and the pages follow the tables.
///
/// The layout for the arena there for would be
///
/// |Table|Table|...|Page|Page|Page|...
///
/// Each table maps its GiB in both the identity half and the direct map
/// at `PHYSICAL_MEMORY_OFFSET`.
pub struct ArenaPlacement {
    tables: Range<usize>,
    pages: Range<usize>,
    /// The GiBs the tables map, one table each.
    gibs: Range<usize>,
}

impl ArenaPlacement {
    /// Places `size` bytes of pages, and the tables that map RAM up to
    /// `ram_end`, at the start of the first usable region at or above
    /// 1 MiB, and maps them.
    ///
    /// # Safety
    ///
    /// Nothing may use the usable memory yet, and the boot map must map
    /// nothing past the first GiB.
    pub unsafe fn find(
        mmap: &MemoryMap,
        size: usize,
        ram_end: usize,
    ) -> ArenaPlacement {
        let gibs = unmapped_gibs(ram_end);
        let placement = mmap
            .regions
            .read()
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .find_map(|r| unsafe {
                ArenaPlacement::at_start_of(r, size, gibs.clone())
            })
            .expect("a usable region holds the page arena");
        // SAFETY: the placement is in usable memory, which the caller
        // promises is unused.
        unsafe { placement.map() };
        placement
    }

    /// The tables for `gibs` and `size` bytes of pages at the start of
    /// `region`'s part at or above 1 MiB, if they fit in it.
    ///
    /// # Safety
    ///
    /// Region must be a useable memory region.
    unsafe fn at_start_of(
        region: &MemoryRegion,
        size: usize,
        gibs: Range<usize>,
    ) -> Option<Self> {
        debug_assert!(region.region_type == MemoryRegionType::Usable);
        let table_start = (region.base_address as usize)
            .max(MiB)
            .next_multiple_of(REGULAR_PAGE_SIZE);
        let end = (region.base_address + region.length) as usize;

        let pages_start =
            table_start + gibs.len() * size_of::<PageTable>();

        let fits = pages_start + size <= end;
        let tables_are_boot_mapped = gibs.is_empty()
            || pages_start <= BOOT_MAPPED_GIBS * HUGE_PAGE_SIZE;

        (fits && tables_are_boot_mapped).then(|| ArenaPlacement {
            tables: table_start..pages_start,
            pages: pages_start..pages_start + size,
            gibs,
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

    /// Maps each GiB in `gibs` with 2 MiB pages, through one carved L2
    /// table per GiB that both the identity L3 and the direct-map L3
    /// point at.
    ///
    /// # Safety
    ///
    /// The placement's memory must be usable and unused, and the boot map
    /// must map nothing past the first GiB.
    unsafe fn map(&self) {
        let l3s = [0, PHYSICAL_MEMORY_OFFSET]
            .map(|base| boot_l3_of(VirtualAddress::from(base)));

        let tables = self.tables.clone().step_by(size_of::<PageTable>());
        for (gib, table) in self.gibs.clone().zip(tables) {
            // SAFETY: `table` is unused memory in the first GiB, which
            // the boot map identity-maps.
            unsafe {
                PhysicalAddress::from(table)
                    .as_non_null::<PageTable>()
                    .write(gib_table(gib))
            };

            for mut l3 in l3s {
                // SAFETY: nothing maps this GiB yet in either half.
                unsafe {
                    l3.as_mut()
                        .entries
                        .get_mut(gib)
                        .expect("RAM ends below 512 GiB")
                        .map_unchecked(
                            PhysicalAddress::from(table),
                            PageEntryFlags::table_flags(),
                        )
                };
            }
        }
    }
}

/// The boot map's L3 table that holds `address`.
fn boot_l3_of(address: VirtualAddress) -> NonNull<PageTable> {
    let pml4 = PageTable::current_table();
    let index = address.index_of(PageTableLevel::PML4);
    // SAFETY: CR3 holds the boot map's live PML4.
    unsafe { pml4.as_ref() }.entries[index]
        .mapped_table()
        .expect("the boot map has an L3 table in each half")
}

/// An L2 table that maps all of GiB `gib` with 2 MiB pages.
fn gib_table(gib: usize) -> PageTable {
    let mut l2 = PageTable::empty();
    for (entry, frame) in l2.entries.iter_mut().zip(big_frames_of(gib)) {
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

/// The GiBs below `ram_end` that the boot map doesn't cover.
fn unmapped_gibs(ram_end: usize) -> Range<usize> {
    BOOT_MAPPED_GIBS
        ..ram_end.div_ceil(HUGE_PAGE_SIZE).max(BOOT_MAPPED_GIBS)
}

/// The 2 MiB frames of GiB `gib`, in order.
fn big_frames_of(gib: usize) -> StepBy<Range<usize>> {
    let start = gib * HUGE_PAGE_SIZE;
    (start..start + HUGE_PAGE_SIZE).step_by(BIG_PAGE_SIZE)
}
