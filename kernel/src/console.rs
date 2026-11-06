use alloc::boxed::Box;
use common::enums::PS2ScanCode;
use keyboard::{KEYBOARD, KEYBOARD_BUFFER, ps2_keyboard::Keyboard};
use page::{Page, arena::PageMap};
use slab::SlabAllocator;
use sync::{mutex::SpinMutex, spsc::SpscRingBuffer};
use vga_display::{
    ADVANCED_WRITER, WRITER, advanced_writer::AdvancedWriter, okprintln,
    print,
};
use x86::instructions::interrupts;

/// Publishes the keyboard and its scan code buffer, so the keyboard
/// handler stops dropping keys.
pub fn init_keyboard(slab: &'static SlabAllocator<Page, PageMap>) {
    let buffer: Box<[u8], _> = Box::new_in([0u8; 4096], slab);
    let buffer_address = buffer.as_ptr();
    let keyboard_buffer =
        KEYBOARD_BUFFER.init(SpscRingBuffer::new(buffer));
    KEYBOARD.init(SpinMutex::new(Keyboard::new(keyboard_buffer)));
    okprintln!("Initialized Keyboard at {:p}", buffer_address);
}

/// Switches printing to the advanced writer, then waits for its first
/// redraw.
///
/// Interrupts must be enabled, because the timer handler does the redraw
/// and wakes the CPU.
pub fn use_advanced_writer() {
    // SAFETY: this is the only `leak` of the writer, and it stays locked.
    let writer = unsafe {
        ADVANCED_WRITER
            .init(SpinMutex::new_locked(AdvancedWriter::default()))
            .leak()
    };
    WRITER.lock().set_writer(writer);
    okprintln!("Set advanced writer");

    unsafe { interrupts::hlt() };
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
        PS2ScanCode::UpArrow => WRITER.lock().inner.scroll_up(1),
        PS2ScanCode::DownArrow => WRITER.lock().inner.scroll_down(1),
        PS2ScanCode::LeftArrow => WRITER.lock().inner.scroll_up(24),
        PS2ScanCode::RightArrow => WRITER.lock().inner.scroll_down(24),
        _ => {}
    }
}
