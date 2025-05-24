pub mod address_types;
pub mod page_tables;

use constants::{
    addresses::{
        IDENTITY_PAGE_TABLE_L2_OFFSET, IDENTITY_PAGE_TABLE_L3_OFFSET,
        IDENTITY_PAGE_TABLE_L4_OFFSET, VGA_BUFFER_PTR,
    },
    values::PAGE_DIRECTORY_ENTRIES,
};
use core::{arch::asm, ops::Add};
use page_tables::PageTableEntry;
#[allow(static_mut_refs)]
use page_tables::{PageEntryFlags, PageTable};
#[allow(static_mut_refs)]
#[cfg(target_arch = "x86")]
pub fn enable() {
    use address_types::PhysicalAddress;
    use constants::{
        addresses::{TOP_IDENTITY_PAGE_TABLE_L2_OFFSET, TOP_IDENTITY_PAGE_TABLE_L3_OFFSET},
        values::BIG_PAGE_SIZE,
    };

    let IDENTITY_PAGE_TABLE_L4 =
        unsafe { PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L4_OFFSET) };
    let IDENTITY_PAGE_TABLE_L3 =
        unsafe { PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L3_OFFSET) };
    let IDENTITY_PAGE_TABLE_L2 =
        unsafe { PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L2_OFFSET) };
    let TOP_IDENTITY_PAGE_TABLE_L3 =
        unsafe { PageTable::empty_from_ptr(TOP_IDENTITY_PAGE_TABLE_L3_OFFSET) };
    let TOP_IDENTITY_PAGE_TABLE_L2 =
        unsafe { PageTable::empty_from_ptr(TOP_IDENTITY_PAGE_TABLE_L2_OFFSET) };

    unsafe {
        // Setup identity paging
        // Mapping address virtual addresses 0x0000000000000000-0x00000000001fffff to the same physical addresses
        // These entries can't be mapped with the map_table function because only after them the Page Table is considered valid
        // At this point in the code, the variable address is it's physical address because paging is not turned on yet.
        IDENTITY_PAGE_TABLE_L4.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(IDENTITY_PAGE_TABLE_L3_OFFSET),
            PageEntryFlags::table_flags(),
        );
        IDENTITY_PAGE_TABLE_L4.entries[256].map_unchecked(
            PhysicalAddress::new_unchecked(TOP_IDENTITY_PAGE_TABLE_L3_OFFSET),
            PageEntryFlags::table_flags(),
        );
        IDENTITY_PAGE_TABLE_L3.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(IDENTITY_PAGE_TABLE_L2_OFFSET),
            PageEntryFlags::table_flags(),
        );
        TOP_IDENTITY_PAGE_TABLE_L3.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(TOP_IDENTITY_PAGE_TABLE_L2_OFFSET),
            PageEntryFlags::table_flags(),
        );
        for (i, (bot_entry, top_entry)) in IDENTITY_PAGE_TABLE_L2
            .entries
            .iter_mut()
            .zip(TOP_IDENTITY_PAGE_TABLE_L2.entries.iter_mut())
            .enumerate()
        {
            bot_entry.map_unchecked(
                PhysicalAddress::new_unchecked(i * BIG_PAGE_SIZE),
                PageEntryFlags::huge_page_flags(),
            );
            top_entry.map_unchecked(
                PhysicalAddress::new_unchecked(i * BIG_PAGE_SIZE),
                PageEntryFlags::huge_page_flags(),
            );
        }
        // Set the page table at cr3 register
        asm!(
            // load the address of the 4th page table to cr3 so the cpu can access it
            "mov eax, {0}",
            "mov cr3, eax",
            const IDENTITY_PAGE_TABLE_L4_OFFSET
        );

        asm!(
            // Enable Physical Address Extension in cr4
            "mov eax, cr4",
            "or eax, 1 << 5",
            "mov cr4, eax",
        );
        asm!(
            // set long mode bit in the Extended Feature Enable Register Model Specific Register (EFER MSR)
            // This register became architectural from amd64 and also adopted by intel, it's number is 0xC0000080
            "mov ecx, 0xC0000080",
            "rdmsr", // read the MSR specified in ecx into eax
            "or eax, 1 << 8",
            "wrmsr", // write what's in eax to the MSR specified in ecx
        );
        asm!("mov eax, cr0", "or eax, 1 << 31", "mov cr0, eax");
    }
}
