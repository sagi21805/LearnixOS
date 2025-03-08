use crate::{
    bios_enums::{Color, Interrupts, Video, VideoModes}, 
    constants::{SCREEN_HEIGHT, SCREEN_WIDTH, VGA_BUFFER_PTR}, second_stage
};
use core::arch::asm;

#[repr(transparent)]
pub struct MinimalWriter;

impl MinimalWriter {

    second_stage! {

        fn print_char(char: u8) {
            unsafe {
                asm!(
                    "mov ah, {0}",
                    "int {1}",
                    const Video::DisplayChar as u8,
                    const Interrupts::VIDEO as u8,
                    in("al") char,
                    options(nostack, nomem)
                );
            }
        }
    
        pub fn print(str: &str) {
            for char in str.bytes() {
                MinimalWriter::print_char(char);
            }
        }
    
    }

    second_stage! {
        pub fn set_vga_text_mode() {
            MinimalWriter::print("Entered Vga mode");
            unsafe {
                asm!(
                    "mov ah, {0}",
                    "mov al, {1}",
                    "int {2}",
                    const Video::SetMode as u8,
                    const VideoModes::VGA_TX_80X25_PB_9X16_PR_720X400 as u8,
                    const Interrupts::VIDEO as u8
                )
            }
        }        
    }
}

#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {

    second_stage! {
        pub const fn new(foreground: Color, background: Color) -> Self {
            Self((background as u8) << 4 | (foreground as u8))
        }
    
        const fn default() -> Self {
            ColorCode((Color::Black as u8) << 4 | (Color::Yellow as u8))
        }
    }

}

impl Clone for ColorCode {
    second_stage! {
        fn clone(&self) -> ColorCode {
            ColorCode(self.0)
        }
    }
}

impl Copy for ColorCode { }


#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {

    second_stage! {
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
}

impl Clone for ScreenChar {
    second_stage! {
        fn clone(&self) -> Self {
            Self {
                ascii_character: self.ascii_character,
                color_code: self.color_code.clone()
            }
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

    second_stage! {

        pub fn new() -> Self {
            MinimalWriter::set_vga_text_mode();
            Self {
                screen: VGA_BUFFER_PTR, // Dangerous and requires careful handling (static mut ref)
                col: 0,
                row: 0,
            }
        }

        pub fn print(&mut self, message: &str, color: ColorCode) {
            for char in message.bytes() {
                unsafe {
                    self.screen.add(
                        self.col + self.row * SCREEN_WIDTH
                    ).write_volatile(ScreenChar::new(char, color));
                }
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
}
