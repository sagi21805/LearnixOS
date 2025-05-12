pub mod address_types;
pub mod page_tables;

use constants::values::BIG_PAGE_SIZE;
use core::arch::asm;
use page_tables::{PageTable, PageTableEntry};
#[cfg(target_arch = "x86")]
pub static mut IDENTITY_PAGE_TABLE_L4: PageTable = PageTable::empty();
#[cfg(target_arch = "x86")]
pub static mut IDENTITY_PAGE_TABLE_L3: PageTable = PageTable::empty();
#[cfg(target_arch = "x86")]
pub static mut IDENTITY_PAGE_TABLE_L2: PageTable = PageTable::empty();

#[allow(static_mut_refs)]
#[cfg(target_arch = "x86")]
pub fn enable() {
    use address_types::PhysicalAddress;
    use page_tables::PageEntryFlags;
    unsafe {
        // Setup identity paging
        // Mapping address virtual addresses 0x0000000000000000-0x00000000001fffff to the same physical addresses
        // These entries can't be mapped with the map_table function because only after them the Page Table is considered valid
        // At this point in the code, the variable address is it's physical address because paging is not turned on yet.
        IDENTITY_PAGE_TABLE_L4.entries[0].map_unchecked(
            PhysicalAddress::new(&IDENTITY_PAGE_TABLE_L3 as *const _ as usize),
            PageEntryFlags::table_flags(),
        );
        IDENTITY_PAGE_TABLE_L3.entries[0].map_unchecked(
            PhysicalAddress::new(&IDENTITY_PAGE_TABLE_L2 as *const _ as usize),
            PageEntryFlags::table_flags(),
        );
        for (i, entry) in IDENTITY_PAGE_TABLE_L2.entries.iter_mut().enumerate() {
            entry.map_unchecked(
                PhysicalAddress::new(i * BIG_PAGE_SIZE),
                PageEntryFlags::huge_page_flags(),
            );
        }

        // Set the page table at cr3 register
        asm!(
            // load the address of the 4th page table to cr3 so the cpu can access it
            "mov eax, {0}",
            "mov cr3, eax",
            in(reg) &IDENTITY_PAGE_TABLE_L4 as *const PageTable
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

        // Enable paging
        asm!("mov eax, cr0", "or eax, 1 << 31", "mov cr0, eax",);
    }
}
