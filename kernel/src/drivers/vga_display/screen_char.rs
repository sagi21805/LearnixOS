use super::ColorCode;

#[repr(C, packed)]
#[derive(Clone)]
pub struct ScreenChar {
    char: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    /// Create a default Screen char with Space as char value, and with the default [`ColorCode`]
    pub const fn default() -> Self {
        Self {
            char: b' ',
            color_code: ColorCode::default(),
        }
    }

    /// Create a new instance with the given char and [`ColorCode`]
    pub const fn new(char: u8, color: ColorCode) -> Self {
        Self {
            char,
            color_code: color,
        }
    }
}

impl Copy for ScreenChar {}
