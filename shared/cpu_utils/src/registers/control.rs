pub mod cr3 {
    use core::arch::asm;

    /// Write `val` to cr3 and return the previous value
    pub fn write(val: usize) -> usize {
        let prev = read();
        if val != prev {
            unsafe {
                asm!(
                    "mov cr3, {}",
                    in(reg) val
                )
            }
        }
        prev
    }

    pub fn read() -> usize {
        unsafe {
            let cr3: usize;
            asm!("mov {}, cr3", out(reg) cr3);
            cr3
        }
    }
}
