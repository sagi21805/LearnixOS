macro_rules! impl_reg_read_write_u64 {
    ($name:ident) => {
        pub mod $name {
            use core::arch::asm;

            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            concat!("mov ", stringify!($name), ", {}"),
                            in(reg) val,
                            options(preserves_flags, nomem, nostack)
                        )
                    }
                }
                prev
            }

            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        concat!("mov {}, ", stringify!($name)),
                        out(reg) val,
                        options(preserves_flags, nomem, nostack)
                    );
                    val
                }
            }
        }
    };
}
macro_rules! impl_reg_read_write_u32 {
    ($name:ident) => {
        pub mod $name {
            use core::arch::asm;

            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        concat!("mov {0:e}, ", stringify!($name)),
                        out(reg) val,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                val
            }

            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        concat!("mov ", stringify!($name), ", {0:e}"),
                        in(reg) val,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                prev
            }
        }
    }
}
macro_rules! impl_reg_read_write_u16 {
    ($name:ident) => {
        pub mod $name {
            use core::arch::asm;

            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        concat!("mov {0:x}, ", stringify!($name)),
                        out(reg) val,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                val
            }

            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        concat!("mov ", stringify!($name), ", {0:x}"),
                        in(reg) val,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                prev
            }
        }
    }
}
macro_rules! impl_reg_read_write_u8 {
    ($name:ident) => {
        pub mod $name {
            use core::arch::asm;

            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        concat!("mov {}, ", stringify!($name)),
                        out(reg_byte) val,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                val
            }

            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        concat!("mov ", stringify!($name), ", {}"),
                        in(reg_byte) val,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                prev
            }
        }
    }
}
pub(crate) use impl_reg_read_write_u8;
pub(crate) use impl_reg_read_write_u16;
pub(crate) use impl_reg_read_write_u32;
pub(crate) use impl_reg_read_write_u64;
