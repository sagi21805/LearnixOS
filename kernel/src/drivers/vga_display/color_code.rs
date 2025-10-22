#[repr(u8)]

/// All the colors coded per the VGA documentation
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
    /// Set the VGA char Background and Foreground
    ///
    /// # Parameters
    ///
    /// - `foreground`: The color of the character itself
    /// - `background`: The background color of the character
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    /// Create a default color code with white characters in
    /// a black background
    pub const fn default() -> Self {
        ColorCode::new(Color::White, Color::Black)
    }
}
