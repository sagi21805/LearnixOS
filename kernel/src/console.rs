use alloc::boxed::Box;
use common::enums::PS2ScanCode;
use keyboard::{KEYBOARD, ps2_keyboard::Keyboard};
use libk::{okprintln, print};
use sync::spsc::SpscRingBuffer;
use vga_display::SCREEN;
use x86::interrupt_safe_mutex::InterruptSafeMutex;

/// Publishes the keyboard, which owns both ends of its scan code buffer.
pub fn init_keyboard() {
    let buffer: Box<[u8]> = Box::new([0u8; 4096]);
    let buffer_address = buffer.as_ptr();
    let ring = Box::leak(Box::new(SpscRingBuffer::new(buffer)));
    KEYBOARD.init(InterruptSafeMutex::new(Keyboard::new(ring)));
    okprintln!("Initialized Keyboard at {:p}", buffer_address);
}

/// Prints what the user types and scrolls on the arrow keys, forever.
///
/// Up and down scroll one line, left and right scroll 24.
pub fn run() -> ! {
    loop {
        match read_input() {
            Ok(text) => print!("{}", text),
            Err(PS2ScanCode::UpArrow) => SCREEN.lock().scroll_view_up(1),
            Err(PS2ScanCode::DownArrow) => {
                SCREEN.lock().scroll_view_down(1)
            }
            Err(PS2ScanCode::LeftArrow) => {
                SCREEN.lock().scroll_view_up(24)
            }
            Err(PS2ScanCode::RightArrow) => {
                SCREEN.lock().scroll_view_down(24)
            }
            Err(_) => {}
        }
    }
}

/// The next typed text, or the key that produced none.
fn read_input() -> Result<&'static str, PS2ScanCode> {
    KEYBOARD.lock().read_char()
}
