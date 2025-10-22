use core::arch::asm;

/// x86/x86_64-only.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn enable() {
    unsafe { asm!("sti", options(nostack, preserves_flags)) };
}

/// x86/x86_64-only.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn disable() {
    unsafe { asm!("cli", options(nostack, preserves_flags)) };
}

/// x86/x86_64-only. Halts the CPU until the next external
/// interrupt.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn hlt() {
    unsafe { asm!("hlt", options(nostack, nomem)) };
}
