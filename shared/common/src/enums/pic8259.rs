pub enum PicCommandCode {
    CascadeMode = 0x1,
    Initialize = 0x10,
    EndOfInterrupt = 0x20,
}

#[derive(Clone, Copy)]
pub enum PicInterruptVectorOffset {
    Master = 0x20,
    Slave = 0x28,
}
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum PicInterruptLine {
    Irq0 = 1 << 0,
    Irq1 = 1 << 1,
    Irq2 = 1 << 2,
    Irq3 = 1 << 3,
    Irq4 = 1 << 4,
    Irq5 = 1 << 5,
    Irq6 = 1 << 6,
    Irq7 = 1 << 7,
}
#[derive(Copy, Clone)]
#[repr(u16)]
pub enum CascadedPicInterruptLine {
    Timer = 1 << 0,
    Keyboard = 1 << 1,
    Irq2 = 1 << 2,
    Irq3 = 1 << 3,
    Irq4 = 1 << 4,
    Irq5 = 1 << 5,
    Irq6 = 1 << 6,
    Irq7 = 1 << 7,
    Irq8 = 1 << 8,
    Irq9 = 1 << 9,
    Irq10 = 1 << 10,
    Irq11 = 1 << 11,
    Irq12 = 1 << 12,
    Irq13 = 1 << 13,
    Irq14 = 1 << 14,
    Irq15 = 1 << 15,
}

pub enum PicMode {
    Mode8086 = 0x1,
    ModeAuto = 0x2,
    ModeBufSlave = 0x8,
    ModeBufMaster = 0xc,
    ModeSpecialFullyNested = 0x10,
}
