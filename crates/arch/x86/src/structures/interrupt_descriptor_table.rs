use core::{marker::PhantomData, mem::offset_of};

use common::{
    address_types::{Address, VirtualAddress},
    enums::{
        ProtectionLevel, Sections,
        interrupts::{GateType, InterruptStackTable},
    },
};

use bitfields::bitfields;

use crate::{
    instructions, registers::rflags::Rflags,
    structures::segments::SegmentSelector,
};

/// Interrupt Descriptor Table structure, with one field per vector in
/// vector order.
///
/// A field's type fixes the signature of its handler, because the CPU
/// pushes an error code only on some vectors, and a handler that disagrees
/// corrupts the stack when it returns.
///
/// ```compile_fail,E0308
/// #![feature(abi_x86_interrupt)]
/// use common::enums::interrupts::GateType;
/// use x86::structures::interrupt_descriptor_table::{
///     InterruptDescriptorTable, InterruptStackFrame,
/// };
///
/// extern "x86-interrupt" fn handler(_: InterruptStackFrame) {}
///
/// let mut table = InterruptDescriptorTable::default();
/// table.breakpoint.set_handler(handler, GateType::Trap);
///
/// // Doesn't compile, because the CPU pushes an error code on page faults:
/// table.page_fault.set_handler(handler, GateType::Interrupt);
/// ```
#[repr(C, align(4096))]
pub struct InterruptDescriptorTable {
    pub division_error: InterruptEntry,
    pub debug: InterruptEntry,
    pub non_maskable_interrupt: InterruptEntry,
    pub breakpoint: InterruptEntry,
    pub overflow: InterruptEntry,
    pub bound_range_exceeded: InterruptEntry,
    pub invalid_opcode: InterruptEntry,
    pub device_not_found: InterruptEntry,
    pub double_fault: InterruptEntryWithErrorCode,
    pub coprocessor_segment_overrun: InterruptEntry,
    pub invalid_tss: InterruptEntryWithErrorCode,
    pub segment_not_present: InterruptEntryWithErrorCode,
    pub stack_segment_fault: InterruptEntryWithErrorCode,
    pub general_protection: InterruptEntryWithErrorCode,
    pub page_fault: InterruptEntryWithErrorCode,
    pub intel_reserved: InterruptEntry,
    pub floating_point_error: InterruptEntry,
    pub alignment_check: InterruptEntryWithErrorCode,
    pub machine_check: InterruptEntry,
    pub simd: InterruptEntry,
    pub virtualization: InterruptEntry,
    pub control_protection: InterruptEntryWithErrorCode,
    _intel_reserved_0x16: [InterruptEntry; 10],
    pub timer: InterruptEntry,
    pub keyboard: InterruptEntry,
    _unused_0x22: [InterruptEntry; 8],
    pub ahci: InterruptEntry,
    _unused_0x2b: [InterruptEntry; 213],
}

const ENTRY_SIZE: usize = size_of::<InterruptEntry>();

// A miscounted reserved array would shift every later vector.
const _: () = {
    use InterruptDescriptorTable as Table;

    assert!(ENTRY_SIZE == 16);
    assert!(size_of::<Table>() == 256 * ENTRY_SIZE);
    assert!(offset_of!(Table, page_fault) == 0xe * ENTRY_SIZE);
    assert!(offset_of!(Table, control_protection) == 0x15 * ENTRY_SIZE);
    assert!(offset_of!(Table, timer) == 0x20 * ENTRY_SIZE);
    assert!(offset_of!(Table, keyboard) == 0x21 * ENTRY_SIZE);
    assert!(offset_of!(Table, ahci) == 0x2a * ENTRY_SIZE);
};

impl InterruptDescriptorTable {
    /// Creates a table where every entry is missing
    pub const fn default() -> Self {
        Self {
            division_error: InterruptEntry::missing(),
            debug: InterruptEntry::missing(),
            non_maskable_interrupt: InterruptEntry::missing(),
            breakpoint: InterruptEntry::missing(),
            overflow: InterruptEntry::missing(),
            bound_range_exceeded: InterruptEntry::missing(),
            invalid_opcode: InterruptEntry::missing(),
            device_not_found: InterruptEntry::missing(),
            double_fault: InterruptEntryWithErrorCode::missing(),
            coprocessor_segment_overrun: InterruptEntry::missing(),
            invalid_tss: InterruptEntryWithErrorCode::missing(),
            segment_not_present: InterruptEntryWithErrorCode::missing(),
            stack_segment_fault: InterruptEntryWithErrorCode::missing(),
            general_protection: InterruptEntryWithErrorCode::missing(),
            page_fault: InterruptEntryWithErrorCode::missing(),
            intel_reserved: InterruptEntry::missing(),
            floating_point_error: InterruptEntry::missing(),
            alignment_check: InterruptEntryWithErrorCode::missing(),
            machine_check: InterruptEntry::missing(),
            simd: InterruptEntry::missing(),
            virtualization: InterruptEntry::missing(),
            control_protection: InterruptEntryWithErrorCode::missing(),
            _intel_reserved_0x16: [const { InterruptEntry::missing() };
                10],
            timer: InterruptEntry::missing(),
            keyboard: InterruptEntry::missing(),
            _unused_0x22: [const { InterruptEntry::missing() }; 8],
            ahci: InterruptEntry::missing(),
            _unused_0x2b: [const { InterruptEntry::missing() }; 213],
        }
    }

    /// Load the IDT with the `lidt` instruction
    ///
    /// # Safety
    /// This function overrides the current IDT, and the loaded GDT must
    /// hold the kernel code segment at `Sections::KernelCode`.
    pub unsafe fn load(&'static self) {
        let idtr = InterruptDescriptorTableRegister {
            limit: (size_of::<Self>() - 1) as u16,
            base: self as *const _ as usize,
        };
        unsafe {
            instructions::lidt(&idtr);
        }
    }
}

/// A handler for a vector on which the CPU pushes no error code.
pub type InterruptHandler = extern "x86-interrupt" fn(InterruptStackFrame);

/// A handler for a vector on which the CPU pushes an error code.
pub type InterruptHandlerWithErrorCode =
    extern "x86-interrupt" fn(InterruptStackFrame, u64);

/// The entry of a vector on which the CPU pushes no error code.
pub type InterruptEntry = InterruptDescriptorTableEntry<InterruptHandler>;

/// The entry of a vector on which the CPU pushes an error code.
pub type InterruptEntryWithErrorCode =
    InterruptDescriptorTableEntry<InterruptHandlerWithErrorCode>;

/// Entry structure in the Interrupt Descriptor Table, whose handler has
/// the signature `F`.
#[repr(C, packed)]
pub struct InterruptDescriptorTableEntry<F> {
    handler_offset_low: u16,
    segment_selector: SegmentSelector,
    ist: InterruptStackTable,
    attributes: InterruptAttributes,
    handler_offset_mid: u16,
    handler_offset_high: u32,
    _reserved: u32,
    handler: PhantomData<F>,
}

impl<F> InterruptDescriptorTableEntry<F> {
    /// An entry the CPU treats as not present. Its gate type stays valid,
    /// so using it raises `#NP` and not `#GP`.
    const fn missing() -> Self {
        Self {
            handler_offset_low: 0,
            segment_selector: SegmentSelector::new(),
            ist: InterruptStackTable::None,
            attributes: InterruptAttributes::new()
                .gate_type(GateType::Interrupt),
            handler_offset_mid: 0,
            handler_offset_high: 0,
            _reserved: 0,
            handler: PhantomData,
        }
    }

    /// A present entry that enters `handler` in kernel code.
    fn new(handler: VirtualAddress, gate_type: GateType) -> Self {
        let handler = handler.as_usize();
        Self {
            handler_offset_low: handler as u16,
            segment_selector: SegmentSelector::new()
                .rpl(ProtectionLevel::Ring0)
                .section(Sections::KernelCode),
            ist: InterruptStackTable::None,
            attributes: InterruptAttributes::new()
                .present(true)
                .dpl(ProtectionLevel::Ring0)
                .gate_type(gate_type),
            handler_offset_mid: (handler >> 16) as u16,
            handler_offset_high: (handler >> 32) as u32,
            _reserved: 0,
            handler: PhantomData,
        }
    }
}

impl InterruptEntry {
    /// Makes this entry present, entering `handler` through a `gate_type`.
    pub fn set_handler(
        &mut self,
        handler: InterruptHandler,
        gate_type: GateType,
    ) {
        *self =
            Self::new(VirtualAddress::from(handler as usize), gate_type);
    }
}

impl InterruptEntryWithErrorCode {
    /// Makes this entry present, entering `handler` through a `gate_type`.
    pub fn set_handler(
        &mut self,
        handler: InterruptHandlerWithErrorCode,
        gate_type: GateType,
    ) {
        *self =
            Self::new(VirtualAddress::from(handler as usize), gate_type);
    }
}

/// Attributes of an interrupts entry, includes type and
/// privilege level
#[bitfields]
struct InterruptAttributes {
    #[flag(flag_type = GateType)]
    gate_type: B4,
    #[flag(rc(0))]
    zero: B1,
    #[flag(flag_type = ProtectionLevel)]
    dpl: B2,
    present: B1,
}

/// IDT register structure
#[repr(C, packed)]
pub struct InterruptDescriptorTableRegister {
    pub limit: u16,
    pub base: usize,
}

/// The interrupt stack frame structure that will be given
/// to each interrupt on the stack
#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtualAddress,
    pub code_segment: usize,
    pub cpu_flags: Rflags,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: usize,
}
