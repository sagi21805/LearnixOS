use super::super::structures::paging::page_tables::PageTable;
use core::arch::asm;

/// Writes a value to the CR3 register and returns the previous value.
///
/// Updates the CPU's page table base register (CR3) to the specified value if it differs from the current value. Returns the previous CR3 value.
///
/// # Examples
///
/// ```
/// let old_cr3 = cr3_write(new_cr3_value);
/// // old_cr3 now holds the previous CR3 register value.
/// ```
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
