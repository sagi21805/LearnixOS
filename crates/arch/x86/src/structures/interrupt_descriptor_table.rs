use common::{
    address_types::VirtualAddress,
    enums::{
        ProtectionLevel, Sections, SystemSegmentType,
        interrupts::{Interrupt, InterruptStackTable, InterruptType},
    },
};
use core::{arch::asm, panic};
use core::{mem::MaybeUninit, ptr};
use macros::bitfields;

/// Global reference into the interrupt table
pub static mut IDT: MaybeUninit<&mut InterruptDescriptorTable> =
    MaybeUninit::uninit();

/// Global TSS segment
pub static TSS: TaskStateSegment = TaskStateSegment::default();

use crate::{
    instructions,
    registers::rflags::Rflags,
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
#[bitfields]
pub struct InterruptAttributes {
    ist: B3,
    #[flag(r)]
    reserved: B5,
    #[flag(flag_type = InterruptType)]
    int_type: B3,
    #[flag(rc(0))]
    zero: B1,
    #[flag(flag_type = ProtectionLevel)]
    dpl: B2,
    present: B1,
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
                base_address
                    .as_non_null::<InterruptDescriptorTable>()
                    .as_ptr(),
                InterruptDescriptorTable {
                    interrupts: [const {
                        InterruptDescriptorTableEntry::missing()
                    }; 256],
                },
            );
            uninit.write(
                base_address
                    .as_non_null::<InterruptDescriptorTable>()
                    .as_mut(),
            );
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
                .dpl(dpl)
                .int_type(handler_type),
            SegmentSelector::default()
                .rpl(ProtectionLevel::Ring0)
                .section(Sections::KernelCode),
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
                .int_type(InterruptType::Fault),
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
