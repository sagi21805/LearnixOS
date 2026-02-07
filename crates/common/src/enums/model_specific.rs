#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum MSR {
    EFER = 0xc0000080,
}

pub enum EFERFlag {
    /// IA32_EFER.SCE
    SyscallEnable = 0,
    /// IA32_EFER.LME
    IA32eModeEnable = 1 << 8,
    /// IA32_EFER.LMA
    IA32eModeActive = 1 << 10,
    /// IA32_EFER.NXE
    ExecuteDisableBitEnable = 1 << 11,
}
