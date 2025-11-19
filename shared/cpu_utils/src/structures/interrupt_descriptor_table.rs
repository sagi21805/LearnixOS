use common::{
    address_types::VirtualAddress,
    enums::{
        ProtectionLevel, SystemSegmentType,
        interrupts::{Interrupt, InterruptStackTable, InterruptType},
    },
};
use core::{arch::asm, panic};
use core::{mem::MaybeUninit, ptr};
use learnix_macros::flag;

/// Global reference into the interrupt table
pub static mut IDT: MaybeUninit<&mut InterruptDescriptorTable> =
    MaybeUninit::uninit();

/// Global TSS segment
pub static TSS: TaskStateSegment = TaskStateSegment::default();

use crate::{
    instructions,
    registers::rflags::rflags,
    structures::{
        global_descriptor_table::{
            GlobalDescriptorTableLong, GlobalDescriptorTableRegister,
            SystemSegmentDescriptor64,
        },
        segments::{SegmentSelector, TaskStateSegment},
    },
};

/// Attributes of an interrupts entry, includes type and
/// privilege level
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct InterruptAttributes(u8);

impl InterruptAttributes {
    pub const fn default() -> Self {
        Self(0)
    }

    flag!(present, 7);

    /// Set the type of the interrupt.
    pub const fn set_type(
        mut self,
        interrupt_type: InterruptType,
    ) -> Self {
        self.0 |= interrupt_type as u8;
        self
    }

    /// Set the privilege level from where this interrupt
    /// can be called.
    pub const fn set_dpl(mut self, dpl: ProtectionLevel) -> Self {
        self.0 |= (dpl as u8) << 5;
        self
    }
}

/// Interrupt Descriptor Table structure
#[repr(C, align(4096))]
pub struct InterruptDescriptorTable {
    interrupts: [InterruptDescriptorTableEntry; 256],
}

impl InterruptDescriptorTable {
    /// Initialize the IDT by loading the TSS into the gdt
    /// and writing default values to all the entries
    ///
    /// # Parameters
    ///
    /// - `uninit`: An uninitialized IDT.
    /// - `base_address`: A virtual address that the IDT will be placed on.
    pub fn init(
        uninit: &'static mut MaybeUninit<&mut Self>,
        base_address: VirtualAddress,
    ) {
        let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> =
            MaybeUninit::uninit();
        let gdt = unsafe {
            asm!(
                "sgdt [{}]",
                in(reg) gdt_register.as_mut_ptr(),
                options(nostack, preserves_flags)
            );

            // Get gdt from it's register.
            &mut *(gdt_register.assume_init().base
                as *mut GlobalDescriptorTableLong)
        };

        if TSS.iomb() < size_of::<TaskStateSegment>() as u16 {
            panic!(
                "I/O maps are not supported, change TSS IOMB into number \
                 larger then 0x68"
            )
        }

        let tss = SystemSegmentDescriptor64::new(
            &TSS as *const _ as u64,
            (size_of::<TaskStateSegment>() - 1) as u32,
            SystemSegmentType::TaskStateSegmentAvailable,
        );

        gdt.load_tss(tss);
        unsafe {
            ptr::write_volatile(
                base_address.as_mut_ptr::<Self>(),
                InterruptDescriptorTable {
                    interrupts: [const {
                        InterruptDescriptorTableEntry::missing()
                    }; 256],
                },
            );
            uninit.write(&mut *base_address.as_mut_ptr::<Self>());
            uninit.assume_init_ref().load();
        }
    }

    /// Load the IDT with the `lidt` instruction
    fn load(&'static self) {
        let idtr = {
            InterruptDescriptorTableRegister {
                limit: (size_of::<Self>() - 1) as u16,
                base: self as *const _ as u64,
            }
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
    /// - `handler_type`: The type of the handler (Fault / Trap)
    pub fn set_interrupt_handler(
        &mut self,
        routine: Interrupt,
        handler_address: VirtualAddress,
        dpl: ProtectionLevel,
        handler_type: InterruptType,
    ) {
        let entry = InterruptDescriptorTableEntry::new(
            handler_address,
            InterruptStackTable::None,
            InterruptAttributes::default()
                .present()
                .set_dpl(dpl)
                .set_type(handler_type),
            SegmentSelector::kernel_code(),
        );
        self.interrupts[routine as usize] = entry;
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
            segment_selector: SegmentSelector::default(),
            ist: InterruptStackTable::None,
            attributes: InterruptAttributes::default()
                .set_type(InterruptType::Fault),
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
    pub cpu_flags: rflags,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: usize,
}
