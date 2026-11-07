use core::panic::PanicInfo;

use vga_display::print_panic_message;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // SAFETY: this is the panic handler.
    unsafe { print_panic_message(format_args!("{}", info)) };
    loop {}
}
