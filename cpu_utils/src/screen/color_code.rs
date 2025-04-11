use constants::enums::Color;

#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    pub const fn default() -> Self {
        ColorCode::new(Color::White, Color::Black)
    }
}

impl Clone for ColorCode {
    fn clone(&self) -> ColorCode {
        ColorCode(self.0)
    }
}

impl Copy for ColorCode {}
