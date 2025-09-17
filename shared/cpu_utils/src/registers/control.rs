#[cfg(target_arch = "x86")]
use crate::registers::macros::impl_reg_read_write_u32;
#[cfg(target_arch = "x86_64")]
use crate::registers::macros::impl_reg_read_write_u64;

#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(cr3);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(cr2);

#[cfg(target_arch = "x86")]
impl_reg_read_write_u32!(cr3);
