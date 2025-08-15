use common::{address_types::VirtualAddress, flag};

pub type InterruptServiceRoutine = extern "x86-interrupt" fn(InterruptStackFrame);
pub type InterruptServiceRoutineWithError =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
pub type PageFaultInterruptServiceRoutine =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: PageFaultErrorCode);
/// Interrupt Table Indices
///
/// These indices were taken directly from intel manual
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

pub enum InterruptGate {
    Fault = 0xe,
    Trap = 0xf,
}
pub enum PageFaultErrorCode {}

#[repr(C)]
pub struct InterruptStackTable(u8);

#[repr(C)]
pub struct InterruptAttributes(u8);
#[repr(C)]
pub struct InterruptDescriptorTableEntry {
    isr_offset_low: u16,
    segment_selector: u16,
    ist: InterruptStackTable,
    attributes: InterruptAttributes,
    isr_offset_mid: u16,
    isr_offset_high: u32,
    zero: u32,
}

#[repr(C)]
pub struct InterruptDescriptorTable {
    interrupts: [InterruptDescriptorTableEntry; 256],
}

#[repr(C, packed(2))]
pub struct InterruptDescriptorTableRegister {
    pub limit: u16,
    pub base: *const InterruptDescriptorTable,
}

#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    instruction_pointer: VirtualAddress,
    code_segment: usize,
    cpu_flags: usize,
    stack_pointer: VirtualAddress,
    stack_segment: usize,
}

impl InterruptDescriptorTableEntry {
    pub fn new(
        isr: InterruptServiceRoutine,
        ist: InterruptStackTable,
        attributes: InterruptAttributes,
    ) -> Self {
        todo!()
    }
}

impl InterruptDescriptorTable {
    pub fn set_interrupt(routine: Interrupt, handler_function: InterruptServiceRoutine) {}

    pub fn set_interrupt_with_error(
        routine: Interrupt,
        handler_function: InterruptServiceRoutineWithError,
    ) {
    }

    pub fn set_page_fault(routine: Interrupt, handler_function: PageFaultInterruptServiceRoutine) {}

    pub fn load(&'static self) {}
}

impl InterruptStackTable {}

impl InterruptAttributes {
    flag!(present, 7);

    pub fn set_gate_type(gate_type: InterruptGate) {}

    pub fn set_dpl(dpl: u8) {}
}
