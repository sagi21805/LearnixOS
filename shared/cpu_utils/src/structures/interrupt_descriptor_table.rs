use common::{
    address_types::VirtualAddress,
    enums::{ProtectionLevel, SystemSegmentType},
    flag,
};
use core::{arch::asm, panic};
use core::{mem::MaybeUninit, ptr};

use crate::structures::{
    global_descriptor_table::{
        GlobalDescriptorTableLong, GlobalDescriptorTableRegister, SystemSegmentDescriptor64,
    },
    segments::{SegmentSelector, TaskStateSegment},
};

pub type InterruptHandlerFunction = extern "x86-interrupt" fn(InterruptStackFrame);
pub type InterruptHandlerFunctionWithError =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
pub type PageFaultHandlerFunction =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: PageFaultErrorCode);

pub static mut IDT: MaybeUninit<&mut InterruptDescriptorTable> = MaybeUninit::uninit();
pub static TSS: TaskStateSegment = TaskStateSegment::new();

pub trait InterruptHandlerType {
    fn as_virtual_address(&self) -> VirtualAddress;
}

macro_rules! impl_handler_type {
    ($t:ty) => {
        impl InterruptHandlerType for $t {
            #[inline]
            fn as_virtual_address(&self) -> VirtualAddress {
                unsafe { VirtualAddress::new_unchecked(self as *const _ as usize) }
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
pub enum PageFaultErrorCode {}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct InterruptAttributes(u8);

impl InterruptAttributes {
    pub const fn new() -> Self {
        Self(0)
    }

    flag!(present, 7);

    pub const fn set_type(mut self, interrupt_type: InterruptType) -> Self {
        self.0 |= interrupt_type as u8;
        self
    }

    pub const fn set_dpl(mut self, dpl: ProtectionLevel) -> Self {
        self.0 |= (dpl as u8) << 5;
        self
    }
}

#[repr(C, packed)]
#[derive(Clone, Debug)]
pub struct InterruptDescriptorTableEntry {
    handler_offset_low: u16,
    segment_selector: SegmentSelector,
    ist: InterruptStackTable,
    attributes: InterruptAttributes,
    handler_offset_mid: u16,
    handler_offset_high: u32,
    zero: u32,
}

#[repr(C, align(4096))]
pub struct InterruptDescriptorTable {
    interrupts: [InterruptDescriptorTableEntry; 256],
}

impl InterruptDescriptorTable {
    pub fn init(uninit: &'static mut MaybeUninit<&mut Self>, base_address: *mut Self) {
        let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> = MaybeUninit::uninit();
        let gdt = unsafe {
            asm!(
                "sgdt [{}]",
                in(reg) gdt_register.as_mut_ptr(),
                options(nostack, preserves_flags)
            );

            // Get gdt from it's register.
            &mut *(gdt_register.assume_init().base as *mut GlobalDescriptorTableLong)
        };

        if TSS.iomb() < size_of::<TaskStateSegment>() as u16 {
            panic!("I/O maps are not supported, change TSS IOMB into number larger then 0x68")
        }

        let tss = SystemSegmentDescriptor64::new(
            &TSS as *const _ as u64,
            size_of::<TaskStateSegment>() as u32,
            SystemSegmentType::TaskStateSegmentAvailable,
        );

        gdt.load_tss(tss);
        unsafe {
            ptr::write_volatile(
                base_address,
                InterruptDescriptorTable {
                    interrupts: [const { InterruptDescriptorTableEntry::missing() }; 256],
                },
            );
            uninit.write(&mut *base_address);
            uninit.assume_init_ref().load();
            asm!("sti")
        }
    }

    fn load(&'static self) {
        let idt_register = {
            InterruptDescriptorTableRegister {
                limit: (size_of::<Self>() - 1) as u16,
                base: self as *const _ as usize,
            }
        };
        unsafe {
            asm!(
                "cli",
                "lidt [{}]",
                in(reg) &idt_register,
                options(readonly, nostack, preserves_flags)
            );
        }
    }

    pub fn set_default_interrupt_handler<F: InterruptHandlerType>(
        &mut self,
        routine: Interrupt,
        handler_function: F,
    ) -> InterruptDescriptorTableEntry {
        let default_entry = InterruptDescriptorTableEntry::new(
            handler_function,
            InterruptStackTable::None,
            InterruptAttributes::new()
                .present()
                .set_dpl(ProtectionLevel::Ring0)
                .set_type(InterruptType::Fault),
            SegmentSelector::kernel_code(),
        );
        self.interrupts[routine as usize] = default_entry.clone();
        default_entry.clone()
    }
}

#[repr(C, packed)]
pub struct InterruptDescriptorTableRegister {
    pub limit: u16,
    pub base: usize,
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
    pub const fn missing() -> Self {
        Self {
            handler_offset_low: 0,
            segment_selector: SegmentSelector::new(),
            ist: InterruptStackTable::None,
            attributes: InterruptAttributes::new(),
            handler_offset_mid: 0,
            handler_offset_high: 0,
            zero: 0,
        }
    }

    pub fn new<F: InterruptHandlerType>(
        handler_function: F,
        ist: InterruptStackTable,
        attributes: InterruptAttributes,
        segment_selector: SegmentSelector,
    ) -> Self {
        // let function_address = handler_function.as_virtual_address().as_usize();
        let handler_offset_low = 0xe1e8; //function_address as u16;
        let handler_offset_mid = 0; //(function_address >> 16) as u16;
        let handler_offset_high = 0; //(function_address >> 32) as u32;
        Self {
            handler_offset_low,
            segment_selector,
            ist,
            attributes,
            handler_offset_mid,
            handler_offset_high,
            zero: 0,
        }
    }
}
