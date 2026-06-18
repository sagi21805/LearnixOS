use common::enums::Color;

#[derive(Clone, Copy)]
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
}

#[rustfmt::skip]
impl const Default for ColorCode {
    fn default() -> Self {
        ColorCode::new(Color::White, Color::Black)
    }
}
