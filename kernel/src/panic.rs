use core::panic::PanicInfo;

use vga_display::CONSOLE;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // SAFETY: this never returns, so a guard this panic interrupted
    // doesn't use the console again.
    unsafe { CONSOLE.force_unlock() };
    CONSOLE.lock().panic_message(format_args!("{}", info));
    loop {}
}
