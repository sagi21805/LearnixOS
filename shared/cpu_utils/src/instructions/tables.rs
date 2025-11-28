use core::arch::asm;

#[cfg(target_arch = "x86_64")]
use crate::structures::{
    global_descriptor_table::{
        GlobalDescriptorTableLong, GlobalDescriptorTableRegister,
    },
    interrupt_descriptor_table::InterruptDescriptorTableRegister,
    segments::SegmentSelector,
};
#[cfg(target_arch = "x86")]
use crate::structures::{
    global_descriptor_table::{
        GlobalDescriptorTableProtected, GlobalDescriptorTableRegister,
    },
    segments::SegmentSelector,
};
use core::mem::MaybeUninit;

/// Load the global descriptor table.
///
/// # Safety
/// This function overrides the current GDT if defined.
pub unsafe fn lgdt(gdtr: &GlobalDescriptorTableRegister) {
    unsafe {
        asm!(
            "lgdt [{}]",
            in(reg) gdtr,
            options(readonly, nostack, preserves_flags)
        );
    }
}

#[cfg(target_arch = "x86_64")]
/// Load the interrupt descriptor table.
///
/// # Safety
/// This function overrides the current IDT if defined.
pub unsafe fn lidt(idtr: &InterruptDescriptorTableRegister) {
    unsafe {
        asm!(
            "lidt [{}]",
            in(reg) idtr,
            options(readonly, nostack, preserves_flags)
        );
    }
}

#[cfg(target_arch = "x86")]
/// Store the content of the global descriptor table
///
/// # Safety
/// There is no way to check if the register has valid data in it, or it is
/// not initialized
pub unsafe fn sgdt() -> &'static GlobalDescriptorTableProtected {
    let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> =
        MaybeUninit::uninit();
    unsafe {
        asm!(
            "sgdt [{}]",
            in(reg) gdt_register.as_mut_ptr(),
            options(nostack, preserves_flags)
        );

        // Get gdt from it's register.
        &mut *(gdt_register.assume_init().base
            as *mut GlobalDescriptorTableProtected)
    }
}

#[cfg(target_arch = "x86_64")]
/// Store the content of the global descriptor table
///
/// # Safety
/// There is no way to check if the register has valid data in it, or it is
/// not initialized
pub unsafe fn sgdt() -> &'static GlobalDescriptorTableLong {
    let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> =
        MaybeUninit::uninit();
    unsafe {
        asm!(
            "sgdt [{}]",
            in(reg) gdt_register.as_mut_ptr(),
            options(nostack, preserves_flags)
        );

        // Get gdt from it's register.
        &mut *(gdt_register.assume_init().base
            as *mut &GlobalDescriptorTableLong)
    }
}

/// Load the task register
///
/// # Safety
/// This function does not check if the segment selector points into a
/// valid SegmentSelector
pub unsafe fn ltr(selector: SegmentSelector) {
    unsafe {
        asm!(
            "ltr {0:x}",
            in(reg) selector.as_u16()
        )
    }
}
