#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum Port {
    KeyboardData = 0x60,
    KeyboardStatus = 0x64,
    MasterPicCmd = 0x20,
    MasterPicData = 0x21,
    SlavePicCmd = 0xA0,
    SlavePicData = 0xA1,
    IOWait = 0x80,
}
