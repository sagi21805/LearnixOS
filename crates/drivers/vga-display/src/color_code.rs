use bitfields::bitfields;
use common::enums::Color;

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
            .foreground(Color::White)
            .background(Color::Black)
    }
}

impl ColorCode {
    /// Applies one parameter of a Select Graphic Rendition sequence.
    ///
    /// Bright backgrounds are ignored, because VGA uses that attribute bit
    /// for blinking.
    ///
    /// ```
    /// use vga_display::color_code::ColorCode;
    ///
    /// let mut color = ColorCode::default();
    /// assert_eq!(u8::from(color), 0x0f);
    ///
    /// color.apply_sgr(31);
    /// color.apply_sgr(44);
    /// assert_eq!(u8::from(color), 0x14);
    ///
    /// color.apply_sgr(91);
    /// assert_eq!(u8::from(color), 0x1c);
    ///
    /// color.apply_sgr(0);
    /// assert_eq!(u8::from(color), 0x0f);
    /// ```
    pub fn apply_sgr(&mut self, param: u8) {
        let index = usize::from(param % 10);
        match param {
            0 => *self = ColorCode::default(),
            30..=37 => self.set_foreground(ANSI_COLORS[index]),
            39 => self.set_foreground(Color::White),
            40..=47 => self.set_background(ANSI_COLORS[index]),
            49 => self.set_background(Color::Black),
            90..=97 => self.set_foreground(ANSI_BRIGHT_COLORS[index]),
            _ => {}
        }
    }
}
