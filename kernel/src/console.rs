use alloc::boxed::Box;
use common::enums::PS2ScanCode;
use keyboard::{KEYBOARD, KEYBOARD_BUFFER, ps2_keyboard::Keyboard};
use libk::{okprintln, print};
use page::{Page, arena::PageMap};
use slab::SlabAllocator;
use sync::{mutex::SpinMutex, spsc::SpscRingBuffer};
use vga_display::{CONSOLE, Scrollback};
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

/// Starts keeping the rows that scroll off the screen.
///
/// This allocates before it locks the console, so nothing below the
/// allocator can deadlock by printing.
pub fn attach_scrollback() {
    let scrollback = Scrollback::default();
    CONSOLE.lock().attach_scrollback(scrollback);
    okprintln!("Attached scrollback");
}

/// Prints what the user types and scrolls on the arrow keys, forever.
pub fn run() -> ! {
    loop {
        match read_input() {
            Ok(text) => print!("{}", text),
            Err(key) => scroll(key),
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

/// Scrolls one line on up and down, and 24 lines on left and right.
fn scroll(key: PS2ScanCode) {
    match key {
        PS2ScanCode::UpArrow => CONSOLE.lock().scroll_view_up(1),
        PS2ScanCode::DownArrow => CONSOLE.lock().scroll_view_down(1),
        PS2ScanCode::LeftArrow => CONSOLE.lock().scroll_view_up(24),
        PS2ScanCode::RightArrow => CONSOLE.lock().scroll_view_down(24),
        _ => {}
    }
}
