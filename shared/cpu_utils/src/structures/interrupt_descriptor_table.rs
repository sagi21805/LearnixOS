use common::{
    address_types::VirtualAddress,
    enums::{
        ProtectionLevel, SystemSegmentType,
        interrupts::{Interrupt, InterruptStackTable, InterruptType},
    },
    flag,
};
use core::{arch::asm, panic};
use core::{mem::MaybeUninit, ptr};

pub static mut IDT: MaybeUninit<&mut InterruptDescriptorTable> = MaybeUninit::uninit();
pub static TSS: TaskStateSegment = TaskStateSegment::new();

use crate::structures::{
    global_descriptor_table::{
        GlobalDescriptorTableLong, GlobalDescriptorTableRegister, SystemSegmentDescriptor64,
    },
    segments::{SegmentSelector, TaskStateSegment},
};

/// Interrupt Table Indices
///
/// These indices were taken directly from intel manual

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
            (size_of::<TaskStateSegment>() - 1) as u32,
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
        }
    }

    fn load(&'static self) {
        let idt_register = {
            InterruptDescriptorTableRegister {
                limit: (size_of::<Self>() - 1) as u16,
                base: self as *const _ as u64,
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

    pub fn set_default_interrupt_handler(
        &mut self,
        routine: Interrupt,
        handler_function: VirtualAddress,
    ) {
        let default_entry = InterruptDescriptorTableEntry::new(
            handler_function,
            InterruptStackTable::None,
            InterruptAttributes::new()
                .present()
                .set_dpl(ProtectionLevel::Ring0)
                .set_type(InterruptType::Fault),
            SegmentSelector::kernel_code(),
        );
        self.interrupts[routine as usize] = default_entry;
    }

    pub fn set_interrupt_handler(
        &mut self,
        routine: Interrupt,
        handler_function: VirtualAddress,
        dpl: ProtectionLevel,
        handler_type: InterruptType,
    ) {
        let entry = InterruptDescriptorTableEntry::new(
            handler_function,
            InterruptStackTable::None,
            InterruptAttributes::new()
                .present()
                .set_dpl(dpl)
                .set_type(handler_type),
            SegmentSelector::kernel_code(),
        );
        self.interrupts[routine as usize] = entry;
    }
}

#[repr(C, packed)]
pub struct InterruptDescriptorTableRegister {
    pub limit: u16,
    pub base: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtualAddress,
    pub code_segment: usize,
    pub cpu_flags: usize,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: usize,
}

impl InterruptDescriptorTableEntry {
    pub const fn missing() -> Self {
        Self {
            handler_offset_low: 0,
            segment_selector: SegmentSelector::new(),
            ist: InterruptStackTable::None,
            attributes: InterruptAttributes::new().set_type(InterruptType::Fault),
            handler_offset_mid: 0,
            handler_offset_high: 0,
            zero: 0,
        }
    }

    pub fn new(
        handler_function: VirtualAddress,
        ist: InterruptStackTable,
        attributes: InterruptAttributes,
        segment_selector: SegmentSelector,
    ) -> Self {
        let handler_offset_low = handler_function.as_usize() as u16;
        let handler_offset_mid = (handler_function.as_usize() >> 16) as u16;
        let handler_offset_high = (handler_function.as_usize() >> 32) as u32;
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
