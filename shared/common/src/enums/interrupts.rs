#[repr(u8)]
pub enum Interrupt {
    DivisionError = 0x0,
    Debug = 0x1,
    NonMaskableInterrupt = 0x2,
    Breakpoint = 0x3,
    Overflow = 0x4,
    BoundRangeExceeded = 0x5,
    InvalidOpcode = 0x6,
    DeviceNotFound = 0x7,
    DoubleFault = 0x8,
    CoprocessorSegmentOverrun = 0x9,
    InvalidTSS = 0xa,
    SegmentNotPresent = 0xb,
    StackSegmentFault = 0xc,
    GeneralProtection = 0xd,
    PageFault = 0xe,
    IntelReserved = 0xf,
    FloatingPointError = 0x10,
    AlignmentCheck = 0x11,
    MachineCheck = 0x12,
    SIMD = 0x13,
    Virtualization = 0x14,
    ControlProtection = 0x15,
    // Interrupts until 0x1f are reserved by Intel.
    Timer = 0x20,
}
#[repr(u8)]
#[derive(Clone, Debug, Copy)]
pub enum InterruptStackTable {
    None = 0,
    IST1 = 1,
    IST2 = 2,
    IST3 = 3,
    IST4 = 4,
    IST5 = 5,
    IST6 = 6,
    IST7 = 7,
}

#[repr(u8)]
pub enum InterruptType {
    Fault = 0xe,
    Trap = 0xf,
}

pub struct PageFaultError(u64);

impl PageFaultError {
    page_flag!(0, not_present, protection_violation);
    page_flag!(1, caused_by_write, caused_by_read);
    page_flag!(2, caused_by_user, caused_by_kernel);
    flag!(reserved_bit_accessed, 3);
    flag!(caused_by_instruction_fetch, 4);
    flag!(caused_by_bad_protection_key, 5);
    flag!(caused_by_shadow_stack_access, 6);
    flag!(no_hlat_translation, 7);
    flag!(sgx_violation, 15);
}
