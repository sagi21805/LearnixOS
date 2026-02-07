pub mod cpuid;
#[cfg(target_arch = "x86_64")]
pub mod interrupts;
pub mod macros;
pub mod port;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod tables;
#[cfg(target_arch = "x86_64")]
pub mod tlb;

pub use tables::*;
