#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod cpuid;
#[cfg(target_arch = "x86_64")]
pub mod interrupts;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod macros;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod port;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod tables;
#[cfg(target_arch = "x86_64")]
pub mod tlb;

pub use tables::*;
