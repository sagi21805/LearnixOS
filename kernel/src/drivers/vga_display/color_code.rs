#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Set the VGA char `Color`
    ///
    /// Colors are coded per VGA documentation in the [`Color`] enum
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    /// Create a default color code with white charaters in a black background
    pub const fn default() -> Self {
        ColorCode::new(Color::White, Color::Black)
    }
}
