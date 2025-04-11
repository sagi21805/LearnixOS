pub mod address_types;
pub mod page_tables;

use address_types::PhysicalAddress;
use core::arch::asm;
use page_tables::PageTable;
pub static mut PAGE_TABLE_L4: PageTable = PageTable::empty();
pub static mut PAGE_TABLE_L3: PageTable = PageTable::empty();
pub static mut PAGE_TABLE_L2: PageTable = PageTable::empty();

#[allow(static_mut_refs)]
#[cfg(target_arch = "x86")]
pub fn enable() {
    unsafe {
        // Setup identity paging
        // Mapping address virtual addresses 0x0000000000000000-0x00000000001fffff to the same physical addresses
        PAGE_TABLE_L4.entries[0].set_frame_address(PAGE_TABLE_L3.address());
        PAGE_TABLE_L3.entries[0].set_frame_address(PAGE_TABLE_L2.address());
        PAGE_TABLE_L2.entries[0].set_frame_address(PhysicalAddress::new(0)); // Start at address 0 
        PAGE_TABLE_L2.entries[0].set_huge_page();

        // Enable paging
        asm!(
            // load the address of the 4th page table to cr3 so the cpu can access it
            "mov eax, {0}",
            "mov cr3, eax",
            in(reg) &PAGE_TABLE_L4 as *const PageTable
        );

        asm!(
            // Enable Physical Address Extention in cr4
            "mov eax, cr4",
            "or eax, 1 << 5",
            "mov cr4, eax",
        );

        asm!(
            // set long mode bit in the Extended Feature Enable Register Model Specific Register (EFER MSR)
            // This register became architectual from amd64 and also adopted by intel, it's number is 0xC0000080
            "mov ecx, 0xC0000080",
            "rdmsr", // read the MSR specified in ecx into eax
            "or eax, 1 << 8",
            "wrmsr", // write what's in eax to the MSR specified in ecx
        );

        asm!("mov eax, cr0", "or eax, 1 << 31", "mov cr0, eax",);
    }
}

// This function will working value only when paging is enabled
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn get_current_page_table() -> &'static mut PageTable {
    let cr3: usize;
    asm!("mov {}, cr3", out(reg) cr3);
    core::mem::transmute::<usize, &'static mut PageTable>(cr3)
}
