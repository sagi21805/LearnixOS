use super::ColorCode;

/// One cell of VGA text memory.
///
/// `glyph` is an index into the font, and any byte is a valid index, so
/// reading a cell back from VGA memory is always sound.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScreenChar {
    pub glyph: u8,
    pub color_code: ColorCode,
}

impl ScreenChar {
    /// Create a new instance with the given glyph and
    /// [`ColorCode`]
    pub const fn new(glyph: u8, color: ColorCode) -> Self {
        Self {
            glyph,
            color_code: color,
        }
    }
}

const impl Default for ScreenChar {
    /// Create a default Screen char with a space as its glyph, and with
    /// the default [`ColorCode`]
    fn default() -> Self {
        Self {
            glyph: b' ',
            color_code: ColorCode::default(),
        }
    }
}
