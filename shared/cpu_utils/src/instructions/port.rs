use common::enums::Port;
use core::arch::asm;

#[inline(always)]
pub unsafe fn outb(port: Port, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port as u16,
            in("al") val,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub unsafe fn outw(port: Port, val: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port as u16,
            in("ax") val,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
pub unsafe fn outl(port: Port, val: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port as u16,
            in("eax") val,
            options(nostack, preserves_flags),
        );
    }
}

/// IN instructions
#[inline(always)]
pub unsafe fn inb(port: Port) -> u8 {
    let mut val: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port as u16,
            out("al") val,
            options(nostack, preserves_flags),
        );
    }
    val
}

#[inline(always)]
pub unsafe fn inw(port: Port) -> u16 {
    let mut val: u16;
    unsafe {
        asm!(
            "in ax, dx",
            in("dx") port as u16,
            out("ax") val,
            options(nostack, preserves_flags),
        );
    }
    val
}

#[inline(always)]
pub unsafe fn inl(port: Port) -> u32 {
    let mut val: u32;
    unsafe {
        asm!(
            "in eax, dx",
            in("dx") port as u16,
            out("eax") val,
            options(nostack, preserves_flags),
        );
    }
    val
}

#[inline(always)]
pub unsafe fn iowait() {
    unsafe { outb(Port::None, 0) };
}
