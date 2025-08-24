macro_rules! impl_reg_read_write {
    ($name:ident) => {
        pub mod $name {
            use core::arch::asm;

            pub fn write(val: usize) -> usize {
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
            /// Auto Generated Newest
            pub fn read() -> usize {
                unsafe {
                    let cr3: usize;
                    asm!(
                        concat!("mov {}, ", stringify!($name)),
                        out(reg) cr3,
                        options(preserves_flags, nomem, nostack)
                    );
                    cr3
                }
            }
        }
    };
}
pub(crate) use impl_reg_read_write;
