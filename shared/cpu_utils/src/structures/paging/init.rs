#[cfg(target_arch = "x86")]
pub fn enable() -> Option<()> {
    use super::{PageEntryFlags, PageTable};
    use common::{
        address_types::PhysicalAddress,
        constants::{
            IDENTITY_PAGE_TABLE_L2_OFFSET, IDENTITY_PAGE_TABLE_L3_OFFSET,
            IDENTITY_PAGE_TABLE_L4_OFFSET,
            TOP_IDENTITY_PAGE_TABLE_L2_OFFSET,
            TOP_IDENTITY_PAGE_TABLE_L3_OFFSET,
        },
    };
    use core::arch::asm;

    // ANCHOR: initialize_page_tables
    // These tables will hold the initial identity mapping
    let identity_page_table_l4 = unsafe {
        PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L4_OFFSET.into())?
    };
    let identity_page_table_l3 = unsafe {
        PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L3_OFFSET.into())?
    };
    let identity_page_table_l2 = unsafe {
        PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L2_OFFSET.into())?
    };
    // ANCHOR_END: initialize_page_tables

    // ANCHOR: initialize_top_page_tables
    // These tables will hold identity mapping for the kernel on the top
    // half of the address space
    let top_identity_page_table_l3 = unsafe {
        PageTable::empty_from_ptr(
            TOP_IDENTITY_PAGE_TABLE_L3_OFFSET.into(),
        )?
    };
    let top_identity_page_table_l2 = unsafe {
        PageTable::empty_from_ptr(
            TOP_IDENTITY_PAGE_TABLE_L2_OFFSET.into(),
        )?
    };
    // ANCHOR_END: initialize_top_page_tables

    // ANCHOR: setup_page_tables
    unsafe {
        // Setup identity paging Mapping address virtual addresses
        // 0x000000-0x1fffff to the same physical addresses.
        identity_page_table_l4.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(IDENTITY_PAGE_TABLE_L3_OFFSET),
            PageEntryFlags::table_flags(),
        );
        identity_page_table_l3.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(IDENTITY_PAGE_TABLE_L2_OFFSET),
            PageEntryFlags::table_flags(),
        );
        identity_page_table_l2.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(0),
            PageEntryFlags::huge_page_flags(),
        );
    }
    // ANCHOR_END: setup_page_tables
    // ANCHOR: setup_top_page_tables
    unsafe {
        // Setup kernel identity paging Mapping at the top half
        // of the address space
        // The kernel is mapped from 0xffff800000000000 to
        // 0xffff8000001fffff to the physical addresses of
        // 0x000000-0x1fffff
        // This mapping will allow the kernel to access physical addresses
        // without any dependency on the current mapping
        identity_page_table_l4.entries[256].map_unchecked(
            PhysicalAddress::new_unchecked(
                TOP_IDENTITY_PAGE_TABLE_L3_OFFSET,
            ),
            PageEntryFlags::table_flags(),
        );
        top_identity_page_table_l3.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(
                TOP_IDENTITY_PAGE_TABLE_L2_OFFSET,
            ),
            PageEntryFlags::table_flags(),
        );
        top_identity_page_table_l2.entries[0].map_unchecked(
            PhysicalAddress::new_unchecked(0),
            PageEntryFlags::huge_io_page_flags(),
        );
    }
    // ANCHOR_END: setup_top_page_tables
    // ANCHOR: set_cr3
    unsafe {
        // Set the page table at cr3 register
        asm!(
            // load the address of the 4th page table to cr3
            // so the cpu can access it
            "mov eax, {0}",
            "mov cr3, eax",
            const IDENTITY_PAGE_TABLE_L4_OFFSET
        );
    }
    // ANCHOR_END: set_cr3
    // ANCHOR: set_cr4
    unsafe {
        asm!(
            // Enable Physical Address Extension (number 5) in cr4
            "mov eax, cr4",
            "or eax, 1 << 5",
            "mov cr4, eax",
        );
    }
    // ANCHOR_END: set_cr4
    // ANCHOR: set_efermsr
    unsafe {
        asm!(
            // set long mode bit (number 8) in the Extended Feature
            // Enable Register Model Specific Register
            // (EFER MSR) This register became
            // architectural from amd64 and also adopted by
            // intel, it's number is 0xC0000080
            "mov ecx, 0xC0000080",
            // read the MSR specified in ecx into eax
            "rdmsr",
            "or eax, 1 << 8",
            // write what's in eax to the MSR specified in ecx
            "wrmsr",
        );
    }
    // ANCHOR_END: set_efermsr
    // ANCHOR: enable_paging
    unsafe {
        // Toggle the paging bit (number 31) in cr0
        asm!("mov eax, cr0", "or eax, 1 << 31", "mov cr0, eax");
    }
    Some(())
    // ANCHOR_END: enable_paging
}
