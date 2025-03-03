use crate::{
    bios_enums::{Color, Interrupts, Video, VideoModes}, 
    constants::{SCREEN_HEIGHT, SCREEN_WIDTH, VGA_BUFFER_PTR}
};
use core::{arch::asm, iter::Scan};

#[repr(transparent)]
pub struct MinimalWriter;

impl MinimalWriter {

    #[link_section = ".screen_minimal"]
    #[cfg(feature = "stage-1-2")]
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

    #[inline(never)]
    #[link_section = ".screen_minimal"]
    #[cfg(feature = "stage-1-2")]
    pub fn print(str: &str) {
        for char in str.bytes() {
            MinimalWriter::print_char(char);
        }
    }

    #[link_section = ".kernel_function"]
    #[cfg(feature = "stage-3")]
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

#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {

    #[link_section = ".kernel_function"]
    #[cfg(feature = "stage-3")]
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }


    #[link_section = ".kernel_function"]
    #[cfg(feature = "stage-3")]
    const fn default() -> Self {
        ColorCode((Color::Black as u8) << 4 | (Color::Yellow as u8))
    }

}

impl Clone for ColorCode {

    #[link_section = ".kernel_function"]
    #[cfg(feature = "stage-3")]
    fn clone(&self) -> ColorCode {
        ColorCode(self.0)
    }
}

impl Copy for ColorCode { }


#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    #[link_section = ".kernel_function"]
    #[cfg(feature = "stage-3")]
    const fn default() -> Self {
        Self {
            ascii_character: b'A',
            color_code: ColorCode::default(),
        }
    }

    #[link_section = ".kernel_function"]
    #[cfg(feature = "stage-3")]
    pub const fn new(character: u8, color: ColorCode) -> Self {
        Self {
            ascii_character: character,
            color_code: color,
        }
    }
}

impl Clone for ScreenChar {

    #[link_section = ".kernel_function"]
    #[cfg(feature = "stage-3")]
    fn clone(&self) -> Self {
        Self {
            ascii_character: self.ascii_character,
            color_code: self.color_code.clone()
        }
    }
}

impl Copy for ScreenChar {

}


pub struct Writer {
    screen: *mut ScreenChar,
    col: usize,
    row: usize,
}

impl Writer {

    #[cfg(feature = "stage-3")]
    #[link_section = ".kernel_function"]
    #[allow(static_mut_refs)]
    pub fn new() -> Self {
        MinimalWriter::set_vga_text_mode();
        Self {
            screen: VGA_BUFFER_PTR, // Dangerous and requires careful handling (static mut ref)
            col: 0,
            row: 0,
        }
    }

    // #[link_section = ".kernel_function"]
    // pub fn print(&mut self, message: &str, color: ColorCode) {
    //     for char in message.bytes() {
    //         self.screen[self.col][self.row] = ScreenChar::new(char, color);
    //         self.col += 1;
    //         if self.col >= SCREEN_WIDTH {
    //             self.col = 0;
    //             self.row += 1;
    //         }
    //         if self.row >= SCREEN_HEIGHT {
    //             self.col = 0;
    //             self.row = 0;
    //         }
    //         // MinimalWriter::print("Entered Vga mode");
    //     }   
    // }
}
