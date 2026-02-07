use common::enums::Color;

// ANCHOR: colorcode
#[derive(Clone, Copy)]
pub struct ColorCode(u8);
// ANCHOR_END: colorcode

// ANCHOR: impl_colorcode
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
// ANCHOR_END: impl_colorcode

// ANCHOR: colorcode_default
impl const Default for ColorCode {
    fn default() -> Self {
        ColorCode::new(Color::White, Color::Black)
    }
}
// ANCHOR_END: colorcode_default
