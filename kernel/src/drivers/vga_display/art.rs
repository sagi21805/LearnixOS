use super::color_code::Color;
use super::screen_char::ScreenChar;
use crate::drivers::vga_display::color_code::ColorCode;

const WIDTH: usize = 80;
const HEIGHT: usize = 25;

const BLUE: ScreenChar = ScreenChar::new(b' ', ColorCode::new(Color::Black, Color::Blue));
const WHITE_ON_BLUE: ScreenChar = ScreenChar::new(b' ', ColorCode::new(Color::White, Color::White));

/// Creates a screen buffer representing a blue screen of death (BSOD) with a centered face pattern.
///
/// The returned buffer is filled with a blue background, and a stylized white face is drawn in the center using a fixed pattern.
///
/// # Returns
/// An array of `ScreenChar` representing the BSOD with a face graphic, sized to fit the screen dimensions.
///
/// # Examples
///
/// ```
/// let buffer = make_bsod_with_face();
/// assert_eq!(buffer.len(), WIDTH * HEIGHT);
/// // The buffer contains a blue background with a white face pattern in the center.
/// ```
pub fn make_bsod_with_face() -> [ScreenChar; WIDTH * HEIGHT] {
    let mut buffer = [BLUE; WIDTH * HEIGHT];

    const FACE_PATTERN: [&[u8]; 11] = [
        b"      *******       ",
        b"   ***       ***    ",
        b"  *             *   ",
        b" *    *     *    *  ",
        b" *               *  ",
        b" *               *  ",
        b" *     *****     *  ",
        b" *    *     *    *  ",
        b"  *             *   ",
        b"   ***       ***    ",
        b"      *******       ",
    ];

    let face_start_row = 2;
    let face_width = FACE_PATTERN[0].len();
    let face_start_col = (WIDTH - face_width) / 2;

    for (dy, row) in FACE_PATTERN.iter().enumerate() {
        for (dx, &ch) in row.iter().enumerate() {
            if ch != b' ' {
                let x = face_start_col + dx;
                let y = face_start_row + dy;
                buffer[y * WIDTH + x] = WHITE_ON_BLUE;
            }
        }
    }

    buffer
}
