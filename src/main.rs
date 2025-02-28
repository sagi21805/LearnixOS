#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
use core::arch::{global_asm, asm};

global_asm!(include_str!("../asm/boot.s"));


#[no_mangle]
pub extern "C" fn print_char(c: u8) {
    let ax = u16::from(c) | 0x0e00;
    unsafe {
        asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") ax);
    }
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn first_stage() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    // let color: u16 = 0x0F00;
    // let hello = "Hello cpp world!";
    // let vga = 0xb8000 as *mut u16;
    // let hello_ptr = hello.as_ptr();
    // unsafe {
    //     for i in 0..16 {
    //         vga.add(i + 80).write(color | *(hello_ptr.add(i)) as u16);        
    //     }
    // }
    for char in "Hello World!".chars() {
        print_char(char as u8);
    }
    loop {}
}

#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail(code: u8) -> ! {
    // print_char(b'!');
    // print_char(code);
    loop {
        // hlt()
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}