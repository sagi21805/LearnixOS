#[repr(u8)]
pub enum Sections {
    Null = 0x0,
    KernelCode = 0x8,
    KernelData = 0x10,
}

// Directly taken from Intel Software developer manual volume 3.
pub enum SystemSegmentType {
    TaskStateSegmentAvailable = 0b1001,
    CallGate = 0b1100,
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}
