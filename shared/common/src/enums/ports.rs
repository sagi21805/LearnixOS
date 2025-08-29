#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum Port {
    KeyboardData = 0x60,
    KeyboardStatus = 0x64,
    PrimaryPicCmd = 0x20,
    PrimaryPicData = 0x21,
    SecondaryPicCmd = 0xA0,
    SecondaryPicData = 0xA1,
    IOWait = 0x80,
}
