use common::enums::MSR;
use core::arch::asm;

pub fn rdmsr(msr: MSR) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr as u32,
            out("eax") low,
            out("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

pub unsafe fn wrmsr(msr: MSR, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr as u32,
            in("eax") low,
            in("edx") high,
            options(nostack, preserves_flags),
        );
    }
}
