use super::super::structures::paging::page_tables::PageTable;
use core::arch::asm;

/// Read cr3 register to obtain the current page table and return a reference to it.
// #[allow(unsafe_op_in_unsafe_fn)]
// #[cfg(target_arch = "x86_64")]
// pub fn get_current_page_table() -> &'static mut PageTable {
//     use common::constants::addresses::PHYSICAL_MEMORY_OFFSET;
//     unsafe { &mut *((cr3_read() + PHYSICAL_MEMORY_OFFSET) as *mut PageTable) }
// }

// #[cfg(target_arch = "x86")]
pub fn get_current_page_table() -> &'static mut PageTable {
    unsafe { &mut *((cr3_read()) as *mut PageTable) }
}

/// Load the identity page table and return the previously loaded cr3 register
// pub fn load_identity() -> usize {
//     cr3_write(IDENTITY_PAGE_TABLE_OFFSET)
// }

/// Write `val` to cr3 and return the previous value
pub fn cr3_write(val: usize) -> usize {
    let prev = cr3_read();
    if val != prev {
        unsafe {
            asm!(
                "mov cr3, {}",
                in(reg) val
            )
        }
    }
    prev
}

pub fn cr3_read() -> usize {
    unsafe {
        let cr3: usize;
        asm!("mov {}, cr3", out(reg) cr3);
        cr3
    }
}
