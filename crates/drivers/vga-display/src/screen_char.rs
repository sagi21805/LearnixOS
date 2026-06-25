use ::core::ascii::Char;

use super::ColorCode;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScreenChar {
    pub char: Char,
    pub color_code: ColorCode,
}

impl ScreenChar {
    /// Create a new instance with the given char and
    /// [`ColorCode`]
    pub const fn new(char: Char, color: ColorCode) -> Self {
        Self {
            char,
            color_code: color,
        }
    }
}

#[rustfmt::skip]
impl const Default for ScreenChar {
    /// Create a default Screen char with Space as char
    /// value, and with the default [`ColorCode`]
    fn default() -> Self {
        Self {
            char: Char::Space,
            color_code: ColorCode::default(),
        }
    }
}
