use core::arch::asm;
pub struct CpuidResult {
    /// EAX register.
    pub eax: u32,
    /// EBX register.
    pub ebx: u32,
    /// ECX register.
    pub ecx: u32,
    /// EDX register.
    pub edx: u32,
}

/// Code directly take from rust because I can't import core::arch::x86_64 yet;
pub unsafe fn __cpuid_count(leaf: u32, sub_leaf: u32) -> CpuidResult {
    let eax;
    let ebx;
    let ecx;
    let edx;

    // LLVM sometimes reserves `ebx` for its internal use, we so we need to use
    // a scratch register for it instead.
    unsafe {
        #[cfg(target_arch = "x86")]
        {
            asm!(
                "mov {0}, ebx",
                "cpuid",
                "xchg {0}, ebx",
                out(reg) ebx,
                inout("eax") leaf => eax,
                inout("ecx") sub_leaf => ecx,
                out("edx") edx,
                options(nostack, preserves_flags),
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            asm!(
                "mov {0:r}, rbx",
                "cpuid",
                "xchg {0:r}, rbx",
                out(reg) ebx,
                inout("eax") leaf => eax,
                inout("ecx") sub_leaf => ecx,
                out("edx") edx,
                options(nostack, preserves_flags),
            );
        }
    }
    CpuidResult { eax, ebx, ecx, edx }
}
