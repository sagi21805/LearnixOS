use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    enums::{
        PageSize, ProtectionLevel,
        interrupts::{Interrupt, InterruptType},
    },
};
use keyboard::keyboard_handler;
use libk::alloc::VirtualAddressMapping;
use vga_display::println;
use x86::{
    registers::cr2,
    structures::{
        interrupt_descriptor_table::{
            InterruptDescriptorTable, InterruptStackFrame,
        },
        paging::PageEntryFlags,
    },
};

use crate::timer::timer_handler;

pub extern "x86-interrupt" fn division_error_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: DivisionError");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn debug_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: Debug");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn non_maskable_interrupt_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: NonMaskableInterrupt");
    panic!("Stack frame: {:#?}", stack_frame);
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
    panic!("Interrupt: Overflow");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: BoundRangeExceeded");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: InvalidOpcode");
    panic!("Stack frame: {:#?}", stack_frame);
    panic!("");
}

pub extern "x86-interrupt" fn device_not_found_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: DeviceNotFound");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn coprocessor_segment_overrun_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: CoprocessorSegmentOverrun");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn intel_reserved_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: IntelReserved");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn floating_point_error_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: FloatingPointError");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn machine_check_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: MachineCheck");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: SIMD");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: Virtualization");
    panic!("Stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("Interrupt: DoubleFault");
    panic!("Stack frame: {:#?}", stack_frame);
    panic!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("Interrupt: Invalself.S");
    panic!("Stack frame: {:#?}", stack_frame);
    panic!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("Interrupt: SegmentNotPresent");
    panic!("Stack frame: {:#?}", stack_frame);
    panic!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("Interrupt: StackSegmentFault");
    panic!("Stack frame: {:#?}", stack_frame);
    panic!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("Interrupt: GeneralProtection");
    // panic!("Stack frame: {:#?}", stack_frame);
    // panic!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let faulting_address =
        unsafe { VirtualAddress::new_unchecked(cr2::read() as usize) };

    let identity_map =
        unsafe { PhysicalAddress::new_unchecked(cr2::read() as usize) };

    faulting_address
        .map(
            identity_map,
            Some(PageEntryFlags::regular_page_flags()),
            PageSize::Regular,
        )
        .expect("Cannot map address");
}

pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("Interrupt: AlignmentCheck");
    panic!("Stack frame: {:#?}", stack_frame);
    panic!("Error code: {:#x}", error_code);
}

pub extern "x86-interrupt" fn control_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("Interrupt: ControlProtection");
    panic!("Stack frame: {:#?}", stack_frame);
    panic!("Error code: {:#x}", error_code);
}

#[extend::ext]
pub impl InterruptDescriptorTable {
    fn init_handlers(&mut self) {
        unsafe {
            self.set_interrupt_handler(
                Interrupt::DivisionError,
                VirtualAddress::new_unchecked(
                    division_error_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::Debug,
                VirtualAddress::new_unchecked(
                    debug_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::NonMaskableInterrupt,
                VirtualAddress::new_unchecked(
                    non_maskable_interrupt_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::Breakpoint,
                VirtualAddress::new_unchecked(
                    breakpoint_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Trap,
            );

            self.set_interrupt_handler(
                Interrupt::Overflow,
                VirtualAddress::new_unchecked(
                    overflow_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Trap,
            );

            self.set_interrupt_handler(
                Interrupt::BoundRangeExceeded,
                VirtualAddress::new_unchecked(
                    bound_range_exceeded_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::InvalidOpcode,
                VirtualAddress::new_unchecked(
                    invalid_opcode_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::DeviceNotFound,
                VirtualAddress::new_unchecked(
                    device_not_found_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::DoubleFault,
                VirtualAddress::new_unchecked(
                    double_fault_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::CoprocessorSegmentOverrun,
                VirtualAddress::new_unchecked(
                    coprocessor_segment_overrun_handler as *const ()
                        as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::InvalidTSS,
                VirtualAddress::new_unchecked(
                    invalid_tss_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::SegmentNotPresent,
                VirtualAddress::new_unchecked(
                    segment_not_present_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::StackSegmentFault,
                VirtualAddress::new_unchecked(
                    stack_segment_fault_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::GeneralProtection,
                VirtualAddress::new_unchecked(
                    general_protection_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::PageFault,
                VirtualAddress::new_unchecked(
                    page_fault_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::FloatingPointError,
                VirtualAddress::new_unchecked(
                    floating_point_error_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::AlignmentCheck,
                VirtualAddress::new_unchecked(
                    alignment_check_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::MachineCheck,
                VirtualAddress::new_unchecked(
                    machine_check_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::SIMD,
                VirtualAddress::new_unchecked(
                    simd_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::Virtualization,
                VirtualAddress::new_unchecked(
                    virtualization_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            self.set_interrupt_handler(
                Interrupt::ControlProtection,
                VirtualAddress::new_unchecked(
                    control_protection_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Fault,
            );

            // TODO: ADD THESE INTERRUPT ON A DIFFERENT OCCASION
            self.set_interrupt_handler(
                Interrupt::Timer,
                VirtualAddress::new_unchecked(
                    timer_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Trap,
            );
            self.set_interrupt_handler(
                Interrupt::Keyboard,
                VirtualAddress::new_unchecked(
                    keyboard_handler as *const () as usize,
                ),
                ProtectionLevel::Ring0,
                InterruptType::Trap,
            );
            // self.set_interrupt_handler(
            //     Interrupt::Ahci,
            //     VirtualAddress::new_unchecked(
            //         ahci_interrupt as *const () as usize,
            //     ),
            //     ProtectionLevel::Ring0,
            //     InterruptType::Trap,
            // );
        }
    }
}
