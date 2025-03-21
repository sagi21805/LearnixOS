use crate::global_descritor_table::GDTR;
use core::arch::asm;

pub fn enter_protected_mode() {
    unsafe {
        asm!(
            "cli",
            "lgdt [{}]",
            "mov eax, cr0",
            "or eax, 1",
            "mov cr0, eax",
            in(reg) &GDTR,
            options(readonly, nostack, preserves_flags)
        )
    }
}
