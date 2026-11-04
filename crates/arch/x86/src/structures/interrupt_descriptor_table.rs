use common::{
    address_types::{Address, VirtualAddress},
    enums::{
        ProtectionLevel, Sections,
        interrupts::{GateType, Interrupt, InterruptStackTable},
    },
};

use bitfields::bitfields;

use crate::{
    instructions, registers::rflags::Rflags,
    structures::segments::SegmentSelector,
};

/// Attributes of an interrupts entry, includes type and
/// privilege level
#[bitfields]
pub struct InterruptAttributes {
    #[flag(flag_type = GateType)]
    gate_type: B4,
    #[flag(rc(0))]
    zero: B1,
    #[flag(flag_type = ProtectionLevel)]
    dpl: B2,
    present: B1,
}

/// Interrupt Descriptor Table structure
#[repr(C, align(4096))]
pub struct InterruptDescriptorTable {
    pub interrupts: [InterruptDescriptorTableEntry; 256],
}

impl InterruptDescriptorTable {
    /// Creates a table where every entry is missing
    pub const fn default() -> Self {
        Self {
            interrupts: [const { InterruptDescriptorTableEntry::missing() };
                256],
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
            base: self as *const _ as u64,
        };
        unsafe {
            instructions::lidt(&idtr);
        }
    }

    /// Set an interrupt handler for a given interrupt
    /// without IST
    ///
    /// # Parameters
    ///
    /// - `routine`: The interrupt handler to set
    /// - `handler_address`: The virtual address to the handler function
    /// - `dpl`: The protection level on the handler entry
    /// - `gate_type`: The type of the gate (Interrupt / Trap)
    pub fn set_interrupt_handler(
        &mut self,
        routine: Interrupt,
        handler_address: VirtualAddress,
        dpl: ProtectionLevel,
        gate_type: GateType,
    ) {
        self.interrupts[routine as usize] =
            InterruptDescriptorTableEntry::new(
                handler_address,
                InterruptStackTable::None,
                InterruptAttributes::new()
                    .present(true)
                    .dpl(dpl)
                    .gate_type(gate_type),
                SegmentSelector::new()
                    .rpl(ProtectionLevel::Ring0)
                    .section(Sections::KernelCode),
            );
    }
}

/// Entry structure in the Interrupt Descriptor Table
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

impl InterruptDescriptorTableEntry {
    /// Default values for an entry to be counted missing
    /// and valid
    pub const fn missing() -> Self {
        Self {
            handler_offset_low: 0,
            segment_selector: SegmentSelector::new(),
            ist: InterruptStackTable::None,
            attributes: InterruptAttributes::new()
                .gate_type(GateType::Interrupt),
            handler_offset_mid: 0,
            handler_offset_high: 0,
            zero: 0,
        }
    }

    /// Create a new IDT entry
    ///
    /// # Parameters
    ///
    /// - `handler_address`: The virtual address of the handler function
    /// - `ist`: The InterruptStackTable index for this entry
    /// - `attributes`: The attributes of the entry
    /// - `segment_selector`: The segment selector that will be loaded to
    ///   CS
    pub fn new(
        handler_address: VirtualAddress,
        ist: InterruptStackTable,
        attributes: InterruptAttributes,
        segment_selector: SegmentSelector,
    ) -> Self {
        let handler_offset_low = handler_address.as_usize() as u16;
        let handler_offset_mid = (handler_address.as_usize() >> 16) as u16;
        let handler_offset_high =
            (handler_address.as_usize() >> 32) as u32;
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

/// IDT register structure
#[repr(C, packed)]
pub struct InterruptDescriptorTableRegister {
    pub limit: u16,
    pub base: u64,
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
