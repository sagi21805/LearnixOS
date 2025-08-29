use crate::registers::macros::{
    impl_reg_read_write_u8, impl_reg_read_write_u16, impl_reg_read_write_u32,
    impl_reg_read_write_u64,
};

// A register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rax);
impl_reg_read_write_u32!(eax);
impl_reg_read_write_u16!(ax);
impl_reg_read_write_u8!(ah);
impl_reg_read_write_u8!(al);

// B register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rbx);
impl_reg_read_write_u32!(ebx);
impl_reg_read_write_u16!(bx);
impl_reg_read_write_u8!(bh);
impl_reg_read_write_u8!(bl);

// C register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rcx);
impl_reg_read_write_u32!(ecx);
impl_reg_read_write_u16!(cx);
impl_reg_read_write_u8!(ch);
impl_reg_read_write_u8!(cl);

// D register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rdx);
impl_reg_read_write_u32!(edx);
impl_reg_read_write_u16!(dx);
impl_reg_read_write_u8!(dh);
impl_reg_read_write_u8!(dl);

// SI Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rsi);
impl_reg_read_write_u32!(esi);
impl_reg_read_write_u16!(si);

// DI Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rdi);
impl_reg_read_write_u32!(edi);
impl_reg_read_write_u16!(di);

// SP Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rsp);
impl_reg_read_write_u32!(esp);
impl_reg_read_write_u16!(sp);

// BP Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(rbp);
impl_reg_read_write_u32!(ebp);
impl_reg_read_write_u16!(bp);

// R8 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r8);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r8d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r8w);

// R9 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r9);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r9d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r9w);

// R10 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r10);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r10d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r10w);

// R11 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r11);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r11d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r11w);

// R12 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r12);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r12d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r12w);

// R13 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r13);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r13d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r13w);

// R14 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r14);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r14d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r14w);

// R15 Register
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u64!(r15);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u32!(r15d);
#[cfg(target_arch = "x86_64")]
impl_reg_read_write_u16!(r15w);
