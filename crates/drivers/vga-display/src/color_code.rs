use common::enums::Color;
use macros::bitfields;

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
