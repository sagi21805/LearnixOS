use common::enums::ProtectionLevel;
use core::arch::asm;
use learnix_macros::flag;

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct rflags(u64);

impl rflags {
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

    flag!(carry, 0);
    flag!(parity, 2);
    flag!(auxiliary, 4);
    flag!(zero, 6);
    flag!(sign, 7);
    flag!(tap, 8);
    flag!(interrupt, 9);
    flag!(direction, 10);
    flag!(overflow, 11);

    /// Set I/O privilege level
    pub fn set_iopl(&mut self, privilege_level: ProtectionLevel) {
        self.0 |= (privilege_level as u64) << 12;
    }

    flag!(nested_task, 14);
    flag!(resume, 16);
    flag!(virtual_8086_mode, 17);
    flag!(alignment_check, 18);
    flag!(virtual_interrupt, 19);
    flag!(virtual_interrupt_pending, 20);
    flag!(cpuid_support, 21);

    pub unsafe fn write(&mut self, flags: Self) {
        // HACK: we mark this function as preserves_flags to
        // prevent Rust from restoring saved flags
        // after the "popf" below.
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
