use crate::registers::macros::{
    impl_reg_read_write_u8, impl_reg_read_write_u16, impl_reg_read_write_u32,
    impl_reg_read_write_u64,
};

// A register
impl_reg_read_write_u64!(rax);
impl_reg_read_write_u32!(eax);
impl_reg_read_write_u16!(ax);
impl_reg_read_write_u8!(ah);
impl_reg_read_write_u8!(al);

// B register
impl_reg_read_write_u64!(rbx);
impl_reg_read_write_u32!(ebx);
impl_reg_read_write_u16!(bx);
impl_reg_read_write_u8!(bh);
impl_reg_read_write_u8!(bl);

// C register
impl_reg_read_write_u64!(rcx);
impl_reg_read_write_u32!(ecx);
impl_reg_read_write_u16!(cx);
impl_reg_read_write_u8!(ch);
impl_reg_read_write_u8!(cl);

// D register
impl_reg_read_write_u64!(rdx);
impl_reg_read_write_u32!(edx);
impl_reg_read_write_u16!(dx);
impl_reg_read_write_u8!(dh);
impl_reg_read_write_u8!(dl);

// SI Register
impl_reg_read_write_u64!(rsi);
impl_reg_read_write_u32!(esi);
impl_reg_read_write_u16!(si);

// DI Register
impl_reg_read_write_u64!(rdi);
impl_reg_read_write_u32!(edi);
impl_reg_read_write_u16!(di);

// SP Register
impl_reg_read_write_u64!(rsp);
impl_reg_read_write_u32!(esp);
impl_reg_read_write_u16!(sp);

// BP Register
impl_reg_read_write_u64!(rbp);
impl_reg_read_write_u32!(ebp);
impl_reg_read_write_u16!(bp);

// R8 Register
impl_reg_read_write_u64!(r8);
impl_reg_read_write_u32!(r8d);
impl_reg_read_write_u16!(r8w);

// R9 Register
impl_reg_read_write_u64!(r9);
impl_reg_read_write_u32!(r9d);
impl_reg_read_write_u16!(r9w);

// R10 Register
impl_reg_read_write_u64!(r10);
impl_reg_read_write_u32!(r10d);
impl_reg_read_write_u16!(r10w);

// R11 Register
impl_reg_read_write_u64!(r11);
impl_reg_read_write_u32!(r11d);
impl_reg_read_write_u16!(r11w);

// R12 Register
impl_reg_read_write_u64!(r12);
impl_reg_read_write_u32!(r12d);
impl_reg_read_write_u16!(r12w);

// R13 Register
impl_reg_read_write_u64!(r13);
impl_reg_read_write_u32!(r13d);
impl_reg_read_write_u16!(r13w);

// R14 Register
impl_reg_read_write_u64!(r14);
impl_reg_read_write_u32!(r14d);
impl_reg_read_write_u16!(r14w);

// R15 Register
impl_reg_read_write_u64!(r15);
impl_reg_read_write_u32!(r15d);
impl_reg_read_write_u16!(r15w);
