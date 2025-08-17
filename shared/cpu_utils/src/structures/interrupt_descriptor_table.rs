use common::{address_types::VirtualAddress, enums::ProtectionLevel, flag};

pub type InterruptHandlerFunction = extern "x86-interrupt" fn(InterruptStackFrame);
pub type InterruptHandlerFunctionWithError =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
pub type PageFaultHandlerFunction =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: PageFaultErrorCode);

pub trait InterruptHandlerType {
    fn as_virtual_address(&self) -> VirtualAddress;
}

macro_rules! impl_handler_type {
    ($t:ty) => {
        impl InterruptHandlerType for $t {
            #[inline]
            fn as_virtual_address(&self) -> VirtualAddress {
                VirtualAddress::new(self as *const _ as usize)
            }
        }
    };
}

impl_handler_type!(InterruptHandlerFunction);
impl_handler_type!(InterruptHandlerFunctionWithError);
impl_handler_type!(PageFaultHandlerFunction);

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

pub struct SegmentSelector(u16);
#[repr(C)]
pub struct InterruptDescriptorTableEntry {
    handler_offset_low: u16,
    segment_selector: SegmentSelector,
    ist: InterruptStackTable,
    attributes: InterruptAttributes,
    handler_offset_mid: u16,
    handler_offset_high: u32,
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
    pub fn new<F: InterruptHandlerType>(
        handler_function: F,
        ist: InterruptStackTable,
        attributes: InterruptAttributes,
    ) -> Self {
        let function_address = handler_function.as_virtual_address().as_usize();
        let handler_offset_low = function_address as u16;
        let handler_offset_mid = (function_address >> 16) as u16;
        let handler_offset_high = (function_address >> 32) as u32;
    }
}

impl InterruptDescriptorTable {
    pub fn set_interrupt_handler<F: InterruptHandlerType>(
        &mut self,
        routine: Interrupt,
        handler_function: F,
    ) {
    }

    pub fn load(&'static self) {}
}

impl InterruptStackTable {}

impl InterruptAttributes {
    flag!(present, 7);

    pub fn set_gate_type(gate_type: InterruptGate) {}

    pub fn set_dpl(dpl: ProtectionLevel) {}
}
