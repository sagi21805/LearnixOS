use bitfields::bitfields;
use common::enums::Color;

const DEFAULT_FOREGROUND: Color = Color::White;
const DEFAULT_BACKGROUND: Color = Color::Black;

/// ANSI color numbers, as VGA colors. ANSI puts red in bit 0 of the color
/// number, and VGA puts blue there.
const ANSI_COLORS: [Color; 8] = [
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Brown,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::LightGray,
];

/// The bright version of each color in [`ANSI_COLORS`].
const ANSI_BRIGHT_COLORS: [Color; 8] = [
    Color::DarkGray,
    Color::LightRed,
    Color::LightGreen,
    Color::Yellow,
    Color::LightBlue,
    Color::Pink,
    Color::LightCyan,
    Color::White,
];

#[bitfields]
pub struct ColorCode {
    #[flag(flag_type = Color)]
    pub foreground: B4,
    #[flag(flag_type = Color)]
    pub background: B4,
}

const impl Default for ColorCode {
    fn default() -> Self {
        ColorCode::new()
            .foreground(DEFAULT_FOREGROUND)
            .background(DEFAULT_BACKGROUND)
    }
}

impl ColorCode {
    /// Applies the parameters of a Select Graphic Rendition sequence, in
    /// order.
    ///
    /// Bright backgrounds are ignored, because VGA uses that attribute bit
    /// for blinking. Parameter 38 or 48 stops the sequence, so the color
    /// number in `38;5;31` doesn't read as red.
    ///
    /// ```
    /// use vga_display::color_code::ColorCode;
    ///
    /// let mut color = ColorCode::default();
    /// assert_eq!(u8::from(color), 0x0f);
    ///
    /// color.apply_sgr(&[31, 44]);
    /// assert_eq!(u8::from(color), 0x14);
    ///
    /// color.apply_sgr(&[91]);
    /// assert_eq!(u8::from(color), 0x1c);
    ///
    /// color.apply_sgr(&[0, 38, 5, 31]);
    /// assert_eq!(u8::from(color), 0x0f);
    /// ```
    pub fn apply_sgr(&mut self, params: &[u16]) {
        for &param in params {
            let index = usize::from(param % 10);
            match param {
                0 => *self = ColorCode::default(),
                30..=37 => self.set_foreground(ANSI_COLORS[index]),
                39 => self.set_foreground(DEFAULT_FOREGROUND),
                40..=47 => self.set_background(ANSI_COLORS[index]),
                49 => self.set_background(DEFAULT_BACKGROUND),
                90..=97 => self.set_foreground(ANSI_BRIGHT_COLORS[index]),
                38 | 48 => return,
                _ => {}
            }
        }
    }
}
