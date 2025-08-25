use core::arch::asm;

#[cfg(target_arch = "x86")]
use crate::structures::global_descriptor_table::GlobalDescriptorTableProtected;
use crate::structures::{
    global_descriptor_table::{GlobalDescriptorTableLong, GlobalDescriptorTableRegister},
    segments::SegmentSelector,
};
use core::mem::MaybeUninit;

pub unsafe fn lgdt(gdtr: &GlobalDescriptorTableRegister) {
    unsafe {
        asm!(
            "lgdt [{}]",
            in(reg) gdtr,
            options(readonly, nostack, preserves_flags)
        );
    }
}

#[cfg(target_arch = "x86")]
pub unsafe fn sgdt() -> &'static GlobalDescriptorTableProtected {
    let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> = MaybeUninit::uninit();
    unsafe {
        asm!(
            "sgdt [{}]",
            in(reg) gdt_register.as_mut_ptr(),
            options(nostack, preserves_flags)
        );

        // Get gdt from it's register.
        &mut *(gdt_register.assume_init().base as *mut GlobalDescriptorTableProtected)
    };
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn sgdt() -> &'static GlobalDescriptorTableLong {
    let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> = MaybeUninit::uninit();
    unsafe {
        asm!(
            "sgdt [{}]",
            in(reg) gdt_register.as_mut_ptr(),
            options(nostack, preserves_flags)
        );

        // Get gdt from it's register.
        &mut *(gdt_register.assume_init().base as *mut &GlobalDescriptorTableLong)
    }
}

pub unsafe fn ltr(selector: SegmentSelector) {
    unsafe {
        asm!(
            "ltr {0:x}",
            in(reg) selector.as_u16()
        )
    }
}
