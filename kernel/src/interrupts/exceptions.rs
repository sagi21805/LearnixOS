use common::{
    address_types::{Address, PhysicalAddress, VirtualAddress},
    enums::{PageSize, interrupts::GateType},
};
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

#[extend::ext(name = ExceptionHandlers)]
pub impl InterruptDescriptorTable {
    /// Registers the handler of every CPU exception.
    fn set_exception_handlers(&mut self) {
        self.division_error
            .set_handler(division_error_handler, GateType::Interrupt);
        self.debug.set_handler(debug_handler, GateType::Interrupt);
        self.non_maskable_interrupt.set_handler(
            non_maskable_interrupt_handler,
            GateType::Interrupt,
        );
        self.breakpoint
            .set_handler(breakpoint_handler, GateType::Trap);
        self.overflow.set_handler(overflow_handler, GateType::Trap);
        self.bound_range_exceeded.set_handler(
            bound_range_exceeded_handler,
            GateType::Interrupt,
        );
        self.invalid_opcode
            .set_handler(invalid_opcode_handler, GateType::Interrupt);
        self.device_not_found
            .set_handler(device_not_found_handler, GateType::Interrupt);
        self.double_fault
            .set_handler(double_fault_handler, GateType::Interrupt);
        self.coprocessor_segment_overrun.set_handler(
            coprocessor_segment_overrun_handler,
            GateType::Interrupt,
        );
        self.invalid_tss
            .set_handler(invalid_tss_handler, GateType::Interrupt);
        self.segment_not_present
            .set_handler(segment_not_present_handler, GateType::Interrupt);
        self.stack_segment_fault
            .set_handler(stack_segment_fault_handler, GateType::Interrupt);
        self.general_protection
            .set_handler(general_protection_handler, GateType::Interrupt);
        self.page_fault
            .set_handler(page_fault_handler, GateType::Interrupt);
        self.floating_point_error.set_handler(
            floating_point_error_handler,
            GateType::Interrupt,
        );
        self.alignment_check
            .set_handler(alignment_check_handler, GateType::Interrupt);
        self.machine_check
            .set_handler(machine_check_handler, GateType::Interrupt);
        self.simd.set_handler(simd_handler, GateType::Interrupt);
        self.virtualization
            .set_handler(virtualization_handler, GateType::Interrupt);
        self.control_protection
            .set_handler(control_protection_handler, GateType::Interrupt);
    }
}

extern "x86-interrupt" fn division_error_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: DivisionError\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    panic!("Interrupt: Debug\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!(
        "Interrupt: NonMaskableInterrupt\nStack frame: {stack_frame:?}"
    );
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame,
) {
    println!("Interrupt: Breakpoint");
    println!("Stack frame: {:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: Overflow\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: BoundRangeExceeded\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: InvalidOpcode\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn device_not_found_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: DeviceNotFound\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn coprocessor_segment_overrun_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!(
        "Interrupt: CoprocessorSegmentOverrun\nStack frame: \
         {stack_frame:?}"
    );
}

extern "x86-interrupt" fn floating_point_error_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: FloatingPointError\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn machine_check_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: MachineCheck\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn simd_handler(stack_frame: InterruptStackFrame) {
    panic!("Interrupt: SIMD\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn virtualization_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("Interrupt: Virtualization\nStack frame: {stack_frame:?}");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Interrupt: DoubleFault\nError code: {error_code:#x}\nStack \
         frame: {stack_frame:?}"
    );
}

extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Interrupt: InvalidTSS\nError code: {error_code:#x}\nStack \
         frame: {stack_frame:?}"
    );
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Interrupt: SegmentNotPresent\nError code: \
         {error_code:#x}\nStack frame: {stack_frame:?}"
    );
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Interrupt: StackSegmentFault\nError code: \
         {error_code:#x}\nStack frame: {stack_frame:?}"
    );
}

extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Interrupt: GeneralProtection\nError code: \
         {error_code:#x}\nStack frame: {stack_frame:?}"
    );
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
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

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Interrupt: AlignmentCheck\nError code: {error_code:#x}\nStack \
         frame: {stack_frame:?}"
    );
}

extern "x86-interrupt" fn control_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Interrupt: ControlProtection\nError code: \
         {error_code:#x}\nStack frame: {stack_frame:?}"
    );
}
