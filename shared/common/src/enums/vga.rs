#[repr(u8)]
pub enum VgaCommand {
    CursorOffsetHigh = 0xE,
    CursorOffsetLow = 0xF,
}
