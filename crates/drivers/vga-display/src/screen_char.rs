use core::ascii::Char;

use crate::color_code::ColorCode;

/// One cell of VGA text memory.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScreenChar {
    pub char: u8,
    pub color_code: ColorCode,
}

impl ScreenChar {
    pub const fn new(char: u8, color: ColorCode) -> Self {
        Self {
            char,
            color_code: color,
        }
    }

    /// Checks if a cell is a space with black background, i.e blank.
    pub fn is_blank(&self) -> bool {
        let empty = Self::default();
        self.char == empty.char
            && u8::from(self.color_code) == u8::from(empty.color_code)
    }
}

const impl Default for ScreenChar {
    /// Create a default Screen char with a space as its glyph, and with
    /// the default [`ColorCode`]
    fn default() -> Self {
        Self {
            char: Char::Space.to_u8(),
            color_code: ColorCode::default(),
        }
    }
}
