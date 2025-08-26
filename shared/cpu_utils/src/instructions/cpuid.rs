use core::arch::asm;

use common::enums::CpuidQuery;
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
pub unsafe fn cpuid(leaf: CpuidQuery, sub_leaf: u32) -> CpuidResult {
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
                inout("eax") leaf as u32 => eax,
                inout("ecx") sub_leaf => ecx,
                out("edx") edx,
                options(nostack, preserves_flags),
            );
        }
    }
    CpuidResult { eax, ebx, ecx, edx }
}

pub fn get_vendor_string() -> [u8; 12] {
    let result = unsafe { cpuid(CpuidQuery::GetVendorString, 0) };
    let mut vendor_string = [0u8; 12];
    vendor_string[0..4].copy_from_slice(&result.ebx.to_le_bytes());
    vendor_string[4..8].copy_from_slice(&result.edx.to_le_bytes());
    vendor_string[8..12].copy_from_slice(&result.ecx.to_le_bytes());
    vendor_string
}
