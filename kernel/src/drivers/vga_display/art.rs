use super::color_code::Color;
use super::screen_char::ScreenChar;
use crate::drivers::vga_display::color_code::ColorCode;

const WIDTH: usize = 80;
const HEIGHT: usize = 25;

const BLUE: ScreenChar = ScreenChar::new(b' ', ColorCode::new(Color::Black, Color::Blue));
const WHITE_ON_BLUE: ScreenChar = ScreenChar::new(b' ', ColorCode::new(Color::White, Color::White));

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
