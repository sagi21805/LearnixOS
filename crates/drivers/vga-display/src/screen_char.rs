use super::ColorCode;

// ANCHOR: screen_char
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ScreenChar {
    char: u8,
    color_code: ColorCode,
}
// ANCHOR_END: screen_char

// ANCHOR: impl_screen_char
impl ScreenChar {
    /// Create a new instance with the given char and
    /// [`ColorCode`]
    pub const fn new(char: u8, color: ColorCode) -> Self {
        Self {
            char,
            color_code: color,
        }
    }
}
// ANCHOR_END: impl_screen_char

// ANCHOR: screen_char_default
impl const Default for ScreenChar {
    /// Create a default Screen char with Space as char
    /// value, and with the default [`ColorCode`]
    fn default() -> Self {
        Self {
            char: b' ',
            color_code: ColorCode::default(),
        }
    }
}
// ANCHOR_END: screen_char_default
