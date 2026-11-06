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
/// Adapted from Rust's core CPUID implementation
/// (MIT/Apache-2.0). Upstream attribution retained; minor
/// adjustment for our no_std layout See LICENSE-MIT and
/// LICENSE-APACHE at the repository root.
pub fn cpuid(query: CpuidQuery) -> CpuidResult {
    let eax;
    let ebx;
    let ecx;
    let edx;

    let query_registers = query.registers();

    // LLVM sometimes reserves `ebx` for its internal use,
    // we so we need to use a scratch register for it
    // instead.
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

/// The CPU vendor's ASCII name, such as `GenuineIntel`.
///
/// Leaf 0 returns it in `EBX`, `EDX`, `ECX`, in that order.
pub fn get_vendor_string() -> [u8; 12] {
    let result = cpuid(CpuidQuery::GetVendorString);
    let mut vendor_string = [0u8; 12];
    vendor_string[0..4].copy_from_slice(&result.ebx.to_le_bytes());
    vendor_string[4..8].copy_from_slice(&result.edx.to_le_bytes());
    vendor_string[8..12].copy_from_slice(&result.ecx.to_le_bytes());
    vendor_string
}

/// The CPU's model name, or `None` if this CPU doesn't report one.
///
/// A leaf above the CPU's maximum doesn't fail. It returns the data of the
/// highest basic leaf, so the maximum has to be checked first.
///
/// ```
/// use x86::instructions::cpuid::get_brand_string;
///
/// if let Some(brand) = get_brand_string() {
///     assert_eq!(brand.as_str(), brand.as_str().trim());
/// }
/// ```
pub fn get_brand_string() -> Option<BrandString> {
    let last_leaf = CpuidQuery::GetBrandStringThird.registers().eax;
    if cpuid(CpuidQuery::GetMaxExtendedLeaf).eax < last_leaf {
        return None;
    }
    let registers = [
        CpuidQuery::GetBrandStringFirst,
        CpuidQuery::GetBrandStringSecond,
        CpuidQuery::GetBrandStringThird,
    ]
    .map(cpuid)
    .into_iter()
    .flat_map(|CpuidResult { eax, ebx, ecx, edx }| [eax, ebx, ecx, edx]);

    let mut brand = [0u8; 48];
    let (chunks, _) = brand.as_chunks_mut::<4>();
    for (bytes, register) in chunks.iter_mut().zip(registers) {
        *bytes = register.to_le_bytes();
    }
    Some(BrandString(brand))
}

/// The raw 48 bytes of the brand string, including its padding.
pub struct BrandString([u8; 48]);

impl BrandString {
    /// The name without the NUL padding after it or the spaces around it.
    ///
    /// `"  Intel(R) Core(TM)\0\0" -> "Intel(R) Core(TM)"`
    pub fn as_str(&self) -> &str {
        let end = self
            .0
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.0.len());
        str::from_utf8(&self.0[..end])
            .expect("CPUID brand strings are ASCII")
            .trim()
    }
}

/// Bits 0-31 are ecx
/// Bits 32-63 are edx
pub struct CpuFeatures(pub u64);

impl Default for CpuFeatures {
    fn default() -> Self {
        let features = cpuid(CpuidQuery::GetCpuFeatures);
        Self(((features.edx as u64) << 32) | (features.ecx as u64))
    }
}

impl CpuFeatures {
    cpu_feature!(
        apic,
        ((CpuFeatureEdx::APIC as u64) << 32).trailing_zeros()
    );
}
