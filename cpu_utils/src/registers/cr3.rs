use super::super::structures::paging::page_tables::PageTable;
use core::arch::asm;

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
