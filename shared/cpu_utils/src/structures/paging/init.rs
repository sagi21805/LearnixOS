#[cfg(target_arch = "x86")]
pub fn enable() -> Option<()> {
    use super::{PageEntryFlags, PageTable};
    use common::{
        address_types::PhysicalAddress,
        constants::{
            IDENTITY_PAGE_TABLE_L2_OFFSET, IDENTITY_PAGE_TABLE_L3_OFFSET,
            IDENTITY_PAGE_TABLE_L4_OFFSET, TOP_IDENTITY_PAGE_TABLE_L2_OFFSET,
            TOP_IDENTITY_PAGE_TABLE_L3_OFFSET,
        },
    };
    use core::arch::asm;
    let identity_page_table_l4 =
        unsafe { PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L4_OFFSET.into())? };
    let identity_page_table_l3 =
        unsafe { PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L3_OFFSET.into())? };
    let identity_page_table_l2 =
        unsafe { PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L2_OFFSET.into())? };
    let top_identity_page_table_l3 =
        unsafe { PageTable::empty_from_ptr(TOP_IDENTITY_PAGE_TABLE_L3_OFFSET.into())? };
    let top_identity_page_table_l2 =
        unsafe { PageTable::empty_from_ptr(TOP_IDENTITY_PAGE_TABLE_L2_OFFSET.into())? };

    unsafe {
        // Setup identity paging
        // Mapping address virtual addresses 0x0000000000000000-0x00000000001fffff to the same physical addresses
        // These entries can't be mapped with the map_table function because only after them the Page Table is considered valid
        // At this point in the code, the variable address is it's physical address because paging is not turned on yet.
        identity_page_table_l4.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(IDENTITY_PAGE_TABLE_L3_OFFSET),
            PageEntryFlags::table_flags(),
        );
        identity_page_table_l4.entries[256].map_unchecked(
            PhysicalAddress::new_unchecked(TOP_IDENTITY_PAGE_TABLE_L3_OFFSET),
            PageEntryFlags::table_flags(),
        );
        identity_page_table_l3.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(IDENTITY_PAGE_TABLE_L2_OFFSET),
            PageEntryFlags::table_flags(),
        );
        top_identity_page_table_l3.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(TOP_IDENTITY_PAGE_TABLE_L2_OFFSET),
            PageEntryFlags::table_flags(),
        );
        identity_page_table_l2.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(0),
            PageEntryFlags::huge_page_flags(),
        );
        top_identity_page_table_l2.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(0),
            PageEntryFlags::huge_page_flags(),
        );
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
    Some(())
}
