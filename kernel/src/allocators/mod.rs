use core::arch::asm;
use cpu_utils::structures::paging::page_tables::PageTable;

// pub mod free_list_allocator;
mod bitmap;
pub mod page_allocator;

/// Read cr3 register to obtain the current page table and return a reference to it.
#[allow(unsafe_op_in_unsafe_fn)]
pub (in self) fn get_current_page_table() -> &'static mut PageTable {
    unsafe {
        let cr3: usize;
        asm!("mov {}, cr3", out(reg) cr3);
        core::mem::transmute::<usize, &'static mut PageTable>(cr3)
    }
}