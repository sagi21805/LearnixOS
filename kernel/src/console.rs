use alloc::boxed::Box;
use common::enums::PS2ScanCode;
use keyboard::{KEYBOARD, KEYBOARD_BUFFER, ps2_keyboard::Keyboard};
use libk::{okprintln, print};
use page::{Page, arena::PageMap};
use slab::SlabAllocator;
use sync::{mutex::SpinMutex, spsc::SpscRingBuffer};
use vga_display::ScrollControl;
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

/// Starts keeping finished rows, so the arrow keys can scroll through
/// them.
///
/// This allocates before it locks the console, so nothing below the
/// allocator can deadlock by printing.
pub fn attach_scrollback() -> ScrollControl {
    let control = vga_display::attach_scrollback(Box::default());
    okprintln!("Attached scrollback");
    control
}

/// Prints what the user types and scrolls on the arrow keys, forever.
pub fn run(control: ScrollControl) -> ! {
    loop {
        match read_input() {
            Ok(text) => print!("{}", text),
            Err(key) => scroll(control, key),
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
fn scroll(control: ScrollControl, key: PS2ScanCode) {
    match key {
        PS2ScanCode::UpArrow => control.scroll_view_up(1),
        PS2ScanCode::DownArrow => control.scroll_view_down(1),
        PS2ScanCode::LeftArrow => control.scroll_view_up(24),
        PS2ScanCode::RightArrow => control.scroll_view_down(24),
        _ => {}
    }
}
