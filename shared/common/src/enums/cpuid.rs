#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum CpuFeatureEcx {
    SSE3 = 1 << 0,
    PCLMUL = 1 << 1,
    DTES64 = 1 << 2,
    MONITOR = 1 << 3,
    DS_CPL = 1 << 4,
    VMX = 1 << 5,
    SMX = 1 << 6,
    EST = 1 << 7,
    TM2 = 1 << 8,
    SSSE3 = 1 << 9,
    CNXT_ID = 1 << 10,
    SDBG = 1 << 11,
    FMA = 1 << 12,
    CX16 = 1 << 13,
    XTPR = 1 << 14,
    PDCM = 1 << 15,
    // bit 16 is reserved
    PCID = 1 << 17,
    DCA = 1 << 18,
    SSE4_1 = 1 << 19,
    SSE4_2 = 1 << 20,
    X2APIC = 1 << 21,
    MOVBE = 1 << 22,
    POPCNT = 1 << 23,
    TSC_DEADLINE = 1 << 24,
    AES = 1 << 25,
    XSAVE = 1 << 26,
    OSXSAVE = 1 << 27,
    AVX = 1 << 28,
    F16C = 1 << 29,
    RDRAND = 1 << 30,
    HYPERVISOR = 1 << 31,
}

#[repr(u32)]
pub enum CpuFeatureEdx {
    FPU = 1 << 0,
    VME = 1 << 1,
    DE = 1 << 2,
    PSE = 1 << 3,
    TSC = 1 << 4,
    MSR = 1 << 5,
    PAE = 1 << 6,
    MCE = 1 << 7,
    CX8 = 1 << 8,
    APIC = 1 << 9,
    // 10 reserved
    SEP = 1 << 11,
    MTRR = 1 << 12,
    PGE = 1 << 13,
    MCA = 1 << 14,
    CMOV = 1 << 15,
    PAT = 1 << 16,
    PSE36 = 1 << 17,
    PSN = 1 << 18,
    CLFSH = 1 << 19,
    // 20 reserved
    DS = 1 << 21,
    ACPI = 1 << 22,
    MMX = 1 << 23,
    FXSR = 1 << 24,
    SSE = 1 << 25,
    SSE2 = 1 << 26,
    SS = 1 << 27,
    HTT = 1 << 28,
    TM = 1 << 29,
    // 30 reserved
    PBE = 1 << 31,
}

pub struct QueryRegisters {
    pub eax: u32,
    pub ecx: u32,
}

pub enum CpuidQuery {
    GetVendorString,
    GetCpuFeatures,
}

impl CpuidQuery {
    pub const fn registers(self) -> QueryRegisters {
        match self {
            CpuidQuery::GetVendorString => {
                QueryRegisters { eax: 0, ecx: 0 }
            }
            CpuidQuery::GetCpuFeatures => {
                QueryRegisters { eax: 1, ecx: 0 }
            }
        }
    }
}
