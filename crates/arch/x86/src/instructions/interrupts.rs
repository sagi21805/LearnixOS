use core::{arch::asm, marker::PhantomData};

use crate::registers::rflags::Rflags;

/// x86/x86_64-only.
///
/// # Safety
/// Enable interrupt, which could lead to undefined behavior
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn enable() {
    unsafe { asm!("sti", options(nostack, preserves_flags)) };
}

/// x86/x86_64-only.
///
/// # Safety
/// Disable interrupts, which could lead into undefined behavior.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn disable() {
    unsafe { asm!("cli", options(nostack, preserves_flags)) };
}

/// x86/x86_64-only.
///
/// # Safety
/// Halts the CPU until the next external interrupt. Could stop the system
/// entirely if interrupts are disabled
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn hlt() { unsafe { asm!("hlt", options(nostack, nomem)) }; }

/// Enables interrupts and halts, with no instruction between, so an
/// interrupt that is already pending wakes the `hlt`.
///
/// # Safety
///
/// Every interrupt that can arrive must have a handler.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub unsafe fn enable_and_hlt() {
    // Without `nomem`, as handlers change memory while the CPU halts.
    unsafe { asm!("sti", "hlt", options(nostack, preserves_flags)) };
}

/// Keeps interrupts off on this CPU until it drops, then turns them back
/// on if they were on before.
///
/// Restoring the earlier state, instead of always running `sti`, lets
/// these nest. An inner one saves "off" and leaves interrupts off when it
/// drops.
///
/// ```no_run
/// use x86::instructions::interrupts::InterruptsDisabled;
///
/// {
///     let _interrupts = InterruptsDisabled::new();
///     // No interrupt handler runs on this CPU here.
/// }
/// // Interrupts are back to how they were before the block.
/// ```
pub struct InterruptsDisabled {
    were_enabled: bool,
    /// `!Send`, as the saved flag belongs to the CPU that saved it.
    _cpu_local: PhantomData<*const ()>,
}

impl InterruptsDisabled {
    /// Turns interrupts off, and remembers whether they were on.
    #[expect(
        clippy::new_without_default,
        reason = "`new` turns interrupts off, which a `Default` value \
                  shouldn't do"
    )]
    pub fn new() -> Self {
        let were_enabled = Rflags::read().is_interrupt();
        // SAFETY: the matching `sti` runs in `Drop`.
        unsafe { disable() };
        Self {
            were_enabled,
            _cpu_local: PhantomData,
        }
    }
}

impl Drop for InterruptsDisabled {
    fn drop(&mut self) {
        if self.were_enabled {
            // SAFETY: interrupts were on when `new` ran.
            unsafe { enable() };
        }
    }
}
