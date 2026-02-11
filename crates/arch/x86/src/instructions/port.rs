use common::enums::Port;
use core::arch::asm;
use extend::ext;

#[ext]
pub impl Port {
    #[inline(always)]
    unsafe fn outb(&mut self, val: u8) {
        unsafe {
            asm!(
                "out dx, al",
                in("dx") *self as u16,
                in("al") val,
                options(nostack, preserves_flags),
            );
        }
    }

    #[inline(always)]
    unsafe fn outw(&mut self, val: u16) {
        unsafe {
            asm!(
                "out dx, ax",
                in("dx") *self as u16,
                in("ax") val,
                options(nostack, preserves_flags),
            );
        }
    }

    #[inline(always)]
    unsafe fn outl(&mut self, val: u32) {
        unsafe {
            asm!(
                "out dx, eax",
                in("dx") *self as u16,
                in("eax") val,
                options(nostack, preserves_flags),
            );
        }
    }

    /// IN instructions
    #[inline(always)]
    unsafe fn inb(&self) -> u8 {
        let mut val: u8;
        unsafe {
            asm!(
                "in al, dx",
                in("dx") *self as u16,
                out("al") val,
                options(nostack, preserves_flags),
            );
        }
        val
    }

    #[inline(always)]
    unsafe fn inw(&self) -> u16 {
        let mut val: u16;
        unsafe {
            asm!(
                "in ax, dx",
                in("dx") *self as u16,
                out("ax") val,
                options(nostack, preserves_flags),
            );
        }
        val
    }

    #[inline(always)]
    unsafe fn inl(&self) -> u32 {
        let mut val: u32;
        unsafe {
            asm!(
                "in eax, dx",
                in("dx") *self as u16,
                out("eax") val,
                options(nostack, preserves_flags),
            );
        }
        val
    }

    #[inline(always)]
    unsafe fn iowait() {
        unsafe {
            Port::IOWait.outb(0);
        };
    }
}
