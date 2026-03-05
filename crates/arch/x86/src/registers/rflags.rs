use common::enums::ProtectionLevel;
use core::arch::asm;
use macros::bitfields;

#[bitfields]
pub struct Rflags {
    carry: B1,
    #[flag(r)]
    reserved1: B1,
    parity: B1,
    #[flag(r)]
    reserved2: B1,
    auxiliary: B1,
    #[flag(r)]
    reserved3: B1,
    zero: B1,
    sign: B1,
    tap: B1,
    interrupt: B1,
    direction: B1,
    overflow: B1,
    #[flag(flag_type = ProtectionLevel)]
    iopl: B2,
    nested_task: B1,
    reserved4: B1,
    resume: B1,
    virtual_8086_mode: B1,
    alignment_check: B1,
    virtual_interrupt: B1,
    virtual_interrupt_pending: B1,
    cpuid_support: B1,
    #[flag(r)]
    reserved5: B41,
}

impl Rflags {
    pub fn read() -> Self {
        let r: u64;
        unsafe {
            asm!(
                "pushfq",
                "pop {}",
                out(reg) r,
                options(nomem, preserves_flags),
            );
        }
        Self(r)
    }

    /// Write the given flags to the cpu flags overriding current flags
    ///
    /// # Safety
    /// Writing custom flags is very risky, and can easily lead into
    /// undefined behavior
    pub unsafe fn write(&mut self, flags: Self) {
        // HACK: we mark this function as preserves_flags although it
        // doesn't to prevent Rust from restoring saved flags
        // after the "popfq" below.
        unsafe {
            asm!(
                "push {}",
                "popfq",
                in(reg) flags.0,
                options(nomem, preserves_flags)
            );
        }
    }
}
