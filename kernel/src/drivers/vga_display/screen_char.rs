use super::ColorCode;

#[repr(C)]
pub struct ScreenChar {
    char: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    const fn default() -> Self {
        Self {
            char: b' ',
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
