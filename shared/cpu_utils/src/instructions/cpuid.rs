use core::arch::asm;

use common::enums::{CpuFeatureEdx, CpuidQuery};

use crate::instructions::macros::cpu_feature;
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

/// Query the cpu about certain parameters
///
/// Adapted from Rust's core CPUID implementation (MIT/Apache-2.0).
/// Upstream attribution retained; minor adjustment for our no_std layout
/// See LICENSE-MIT and LICENSE-APACHE at the repository root.
pub unsafe fn cpuid(query: CpuidQuery) -> CpuidResult {
    let eax;
    let ebx;
    let ecx;
    let edx;

    let query_registers = query.registers();

    // LLVM sometimes reserves `ebx` for its internal use, we so we need to use
    // a scratch register for it instead.
    unsafe {
        #[cfg(target_arch = "x86")]
        {
            asm!(
                "mov {0:e}, ebx",
                "cpuid",
                "xchg {0:e}, ebx",
                out(reg) ebx,
                inout("eax") query_registers.eax => eax,
                inout("ecx") query_registers.ecx => ecx,
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
                inout("eax") query_registers.eax => eax,
                inout("ecx") query_registers.ecx => ecx,
                out("edx") edx,
                options(nostack, preserves_flags),
            );
        }
    }
    CpuidResult { eax, ebx, ecx, edx }
}

pub fn get_vendor_string() -> [u8; 12] {
    let result = unsafe { cpuid(CpuidQuery::GetVendorString) };
    let mut vendor_string = [0u8; 12];
    vendor_string[0..4].copy_from_slice(&result.ebx.to_le_bytes());
    vendor_string[4..8].copy_from_slice(&result.edx.to_le_bytes());
    vendor_string[8..12].copy_from_slice(&result.ecx.to_le_bytes());
    vendor_string
}

/// Bits 0-31 are ecx
/// Bits 32-63 are edx
pub struct CpuFeatures(pub u64);

impl CpuFeatures {
    pub fn new() -> Self {
        let features = unsafe { cpuid(CpuidQuery::GetCpuFeatures) };
        Self(((features.edx as u64) << 32) | (features.ecx as u64))
    }

    cpu_feature!(
        apic,
        (((CpuFeatureEdx::APIC as u64) << 32) as u64).trailing_zeros()
    );
}
