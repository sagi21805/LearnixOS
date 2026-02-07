use common::address_types::VirtualAddress;
use core::arch::asm;

use crate::registers::cr3;

pub fn flash_address(address: VirtualAddress) {
    unsafe {
        asm!("invlpg [{0:r}]",
            in(reg) address.as_usize(),
            options(nostack, preserves_flags)
        )
    }
}

pub fn flash_all() {
    let cr3 = cr3::read();
    let _ = cr3::write(cr3);
}

// TODO: Implement pcid when processes are a thing.
