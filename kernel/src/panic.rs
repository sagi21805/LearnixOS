use core::panic::PanicInfo;

use vga_display::{SCREEN, SIMPLE_WRITER};

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        SCREEN.force_unlock();
        SIMPLE_WRITER.force_unlock();
    };
    SIMPLE_WRITER
        .lock()
        .panic_message(format_args!("{}", _info));
    loop {}
}
