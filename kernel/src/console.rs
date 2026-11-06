use alloc::boxed::Box;
use common::enums::PS2ScanCode;
use keyboard::{KEYBOARD, KEYBOARD_BUFFER, ps2_keyboard::Keyboard};
use libk::{okprintln, print};
use page::{Page, arena::PageMap};
use slab::SlabAllocator;
use sync::{mutex::SpinMutex, spsc::SpscRingBuffer};
use x86::instructions::interrupts;

/// Publishes the keyboard and its scan code buffer.
pub fn init_keyboard(slab: &'static SlabAllocator<Page, PageMap>) {
    let buffer: Box<[u8], _> = Box::new_in([0u8; 4096], slab);
    let buffer_address = buffer.as_ptr();
    let keyboard_buffer =
        KEYBOARD_BUFFER.init(SpscRingBuffer::new(buffer));
    KEYBOARD.init(SpinMutex::new(Keyboard::new(keyboard_buffer)));
    okprintln!("Initialized Keyboard at {:p}", buffer_address);
}

/// Prints what the user types, forever.
pub fn run() -> ! {
    loop {
        if let Ok(text) = read_input() {
            print!("{}", text);
        }
    }
}

/// The next typed text, or the key that produced none.
fn read_input() -> Result<&'static str, PS2ScanCode> {
    // SAFETY: the keyboard handler takes the same lock, so it must not run
    // while this function holds it.
    unsafe { interrupts::disable() };
    let input = KEYBOARD.lock().read_char();
    unsafe { interrupts::enable() };
    input
}
