use crate::{
    constants::{SCREEN_HEIGHT, SCREEN_WIDTH, VGA_BUFFER_PTR},
    enums::{Color, Interrupts, Video, VideoModes},
};
use core::arch::asm;

#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    const fn default() -> Self {
        ColorCode((Color::Black as u8) << 4 | (Color::Yellow as u8))
    }
}

impl Clone for ColorCode {
    fn clone(&self) -> ColorCode {
        ColorCode(self.0)
    }
}

impl Copy for ColorCode {}

#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    const fn default() -> Self {
        Self {
            ascii_character: b'A',
            color_code: ColorCode::default(),
        }
    }

    pub const fn new(character: u8, color: ColorCode) -> Self {
        Self {
            ascii_character: character,
            color_code: color,
        }
    }
}

impl Clone for ScreenChar {
    fn clone(&self) -> Self {
        Self {
            ascii_character: self.ascii_character,
            color_code: self.color_code.clone(),
        }
    }
}

impl Copy for ScreenChar {}

pub struct Writer {
    screen: *mut ScreenChar,
    col: usize,
    row: usize,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            screen: VGA_BUFFER_PTR, // Dangerous and requires careful handling (static mut ref)
            col: 0,
            row: 0,
        }
    }

    pub fn write_char(&mut self, char: ScreenChar) {
        unsafe {
            self.screen
                .add(self.col + self.row * SCREEN_WIDTH)
                .write_volatile(char);
        }
    }

    pub fn print(&mut self, message: &str, color: ColorCode) {
        for char in message.bytes() {
            self.write_char(ScreenChar::new(char, color));
            self.col += 1;
            if self.col >= SCREEN_WIDTH {
                self.col = 0;
                self.row += 1;
            }
            if self.row >= SCREEN_HEIGHT {
                self.col = 0;
                self.row = 0;
            }
            // MinimalWriter::print("Entered Vga mode");
        }
    }
}
