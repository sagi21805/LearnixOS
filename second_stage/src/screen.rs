#![allow(unsafe_op_in_unsafe_fn)]
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH, VGA_BUFFER_PTR};
use enums::Color::{*, self};
pub static mut WRITER: Writer = Writer::new(ColorCode::default());

#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    pub const fn default() -> Self {
        ColorCode::new(White, Black)
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
    char: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    const fn default() -> Self {
        Self {
            char: b'A',
            color_code: ColorCode::default(),
        }
    }

    pub const fn new(char: u8, color: ColorCode) -> Self {
        Self {
            char,
            color_code: color,
        }
    }
}

impl Clone for ScreenChar {
    fn clone(&self) -> Self {
        Self {
            char: self.char,
            color_code: self.color_code.clone(),
        }
    }
}

impl Copy for ScreenChar {}

pub struct Writer {
    col: usize,
    row: usize,
    pub color: ColorCode,
}

impl Writer {
    const fn new(color: ColorCode) -> Self {
        Self {
            color,
            col: 0,
            row: 0,
        }
    }

    fn write_char(&mut self, char: u8) {
        unsafe {
            VGA_BUFFER_PTR
                .add(self.col + self.row * SCREEN_WIDTH)
                .write_volatile(ScreenChar::new(char, self.color));
        }
    }

}

impl core::fmt::Write for Writer {

    /// Print the messege in the given color
    /// IMPORTANT: THIS FUNCTION IS NOT THREAD SAFE!
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for char in s.bytes() {
            match char {
                b'\n' => {
                    self.row += 1;
                    self.col = 0;
                } 
                _ => {
                    self.write_char(char);
                    self.col += 1;
                }
            }
            if self.col >= SCREEN_WIDTH {
                self.col = 0;
                self.row += 1;
            }
            if self.row >= SCREEN_HEIGHT {
                self.col = 0;
                self.row = 0;
            }
        }
        Ok(())
    }

}

pub trait ColorAble {
    fn color(self, color: ColorCode) -> Self where Self: Sized {
        unsafe { WRITER.color = color; }
        self
    }
}
impl ColorAble for &str {}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        unsafe {
            write!($crate::WRITER, $($arg)*).unwrap();
            $crate::WRITER.color = ColorCode::default();
        }
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        ($crate::print!("{}\n", format_args!($($arg)*)));
    };
}

// #[macro_export]
// macro_rules! print_colored {
//     ($($arg:tt)+, $color:expr) => {
//         unsafe {
//             $crate::WRITER.color = $color;
//             write!($crate::WRITER, $($arg)*).unwrap() 
//         }
//     };
// }


// #[macro_export]
// macro_rules! println_colored {
//     () => ($crate::print!("\n"));
//     ($color:expr, $($arg:tt)*) => {
//         ($crate::print_colored!($color, "{}\n", format_args!($($arg)*)));
//     };
// }