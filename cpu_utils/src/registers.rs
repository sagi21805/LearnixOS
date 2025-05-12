use constants::addresses::IDENTITY_PAGE_TABLE_OFFSET;

#[cfg(feature = "paging")]
use super::structures::paging::page_tables::PageTable;
use core::arch::asm;

#[cfg(feature = "paging")]
/// Read cr3 register to obtain the current page table and return a reference to it.
#[allow(unsafe_op_in_unsafe_fn)]
pub fn get_current_page_table() -> &'static mut PageTable {
    unsafe { core::mem::transmute::<usize, &'static mut PageTable>(cr3_read()) }
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
