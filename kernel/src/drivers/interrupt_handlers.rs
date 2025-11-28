use crate::{
    drivers::{keyboard::keyboard_handler, timer::timer_handler},
    println,
};
use common::{
    address_types::VirtualAddress,
    enums::{
        ProtectionLevel,
        interrupts::{Interrupt, InterruptType},
    },
};
use cpu_utils::{
    registers::cr2,
    structures::interrupt_descriptor_table::{
        InterruptDescriptorTable, InterruptStackFrame,
    },
};

pub extern "x86-interrupt" fn division_error_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: DivisionError");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn debug_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: Debug");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn non_maskable_interrupt_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: NonMaskableInterrupt");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: Breakpoint");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: Overflow");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: BoundRangeExceeded");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: InvalidOpcode");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn device_not_found_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: DeviceNotFound");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn coprocessor_segment_overrun_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: CoprocessorSegmentOverrun");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn intel_reserved_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: IntelReserved");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn floating_point_error_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: FloatingPointError");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn machine_check_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: MachineCheck");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: SIMD");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: Virtualization");
    println!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: DoubleFault");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: InvalidTSS");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: SegmentNotPresent");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: StackSegmentFault");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: GeneralProtection");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: PageFault");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
    println!("Faulting address: {:x}", cr2::read());
}

pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: AlignmentCheck");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn control_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("Interrupt: ControlProtection");
    println!("Stack frame: {:#?}", stack_frame);
    println!("Error code: {:#x}", error_code);
}

pub fn init(idt: &'static mut InterruptDescriptorTable) {
    unsafe {
        idt.set_interrupt_handler(
            Interrupt::DivisionError,
            VirtualAddress::new_unchecked(
                division_error_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::Debug,
            VirtualAddress::new_unchecked(
                debug_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::NonMaskableInterrupt,
            VirtualAddress::new_unchecked(
                non_maskable_interrupt_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::Breakpoint,
            VirtualAddress::new_unchecked(
                breakpoint_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Trap,
        );

        idt.set_interrupt_handler(
            Interrupt::Overflow,
            VirtualAddress::new_unchecked(
                overflow_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Trap,
        );

        idt.set_interrupt_handler(
            Interrupt::BoundRangeExceeded,
            VirtualAddress::new_unchecked(
                bound_range_exceeded_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::InvalidOpcode,
            VirtualAddress::new_unchecked(
                invalid_opcode_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::DeviceNotFound,
            VirtualAddress::new_unchecked(
                device_not_found_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::DoubleFault,
            VirtualAddress::new_unchecked(
                double_fault_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::CoprocessorSegmentOverrun,
            VirtualAddress::new_unchecked(
                coprocessor_segment_overrun_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::InvalidTSS,
            VirtualAddress::new_unchecked(
                invalid_tss_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::SegmentNotPresent,
            VirtualAddress::new_unchecked(
                segment_not_present_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::StackSegmentFault,
            VirtualAddress::new_unchecked(
                stack_segment_fault_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::GeneralProtection,
            VirtualAddress::new_unchecked(
                general_protection_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::PageFault,
            VirtualAddress::new_unchecked(
                page_fault_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::FloatingPointError,
            VirtualAddress::new_unchecked(
                floating_point_error_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::AlignmentCheck,
            VirtualAddress::new_unchecked(
                alignment_check_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::MachineCheck,
            VirtualAddress::new_unchecked(
                machine_check_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::SIMD,
            VirtualAddress::new_unchecked(
                simd_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::Virtualization,
            VirtualAddress::new_unchecked(
                virtualization_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::ControlProtection,
            VirtualAddress::new_unchecked(
                control_protection_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Fault,
        );

        idt.set_interrupt_handler(
            Interrupt::Timer,
            VirtualAddress::new_unchecked(
                timer_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Trap,
        );
        idt.set_interrupt_handler(
            Interrupt::Keyboard,
            VirtualAddress::new_unchecked(
                keyboard_handler as *const () as usize,
            ),
            ProtectionLevel::Ring0,
            InterruptType::Trap,
        );
    }
}
