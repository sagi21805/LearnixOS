#![feature(prelude_import)]
#![no_std]
#![feature(macro_metavar_expr_concat)]
#![feature(stmt_expr_attributes)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(iter_map_windows)]
extern crate core;
#[prelude_import]
use core::prelude::rust_2024::*;
pub mod instructions {
    pub mod cpuid {
        use core::arch::asm;
        use common::enums::{CpuFeatureEdx, CpuidQuery};
        use crate::instructions::macros::cpu_feature;
        pub struct CpuidResult {
            /// EAX register.
            pub eax: u32,
            /// EBX register.
            pub ebx: u32,
            /// ECX register.
            pub ecx: u32,
            /// EDX register.
            pub edx: u32,
        }
        /// Query the cpu about certain parameters
        ///
        /// Adapted from Rust's core CPUID implementation
        /// (MIT/Apache-2.0). Upstream attribution retained; minor
        /// adjustment for our no_std layout See LICENSE-MIT and
        /// LICENSE-APACHE at the repository root.
        pub fn cpuid(query: CpuidQuery) -> CpuidResult {
            let eax;
            let ebx;
            let ecx;
            let edx;
            let query_registers = query.registers();
            unsafe {
                {
                    asm!(
                        "mov {0:r}, rbx\ncpuid\nxchg {0:r}, rbx", out(reg) ebx,
                        inout("eax") query_registers.eax => eax, inout("ecx")
                        query_registers.ecx => ecx, out("edx") edx,
                        options(preserves_flags, nostack)
                    );
                }
            }
            CpuidResult { eax, ebx, ecx, edx }
        }
        pub fn get_vendor_string() -> [u8; 12] {
            let result = cpuid(CpuidQuery::GetVendorString);
            let mut vendor_string = [0u8; 12];
            vendor_string[0..4].copy_from_slice(&result.ebx.to_le_bytes());
            vendor_string[4..8].copy_from_slice(&result.edx.to_le_bytes());
            vendor_string[8..12].copy_from_slice(&result.ecx.to_le_bytes());
            vendor_string
        }
        /// Bits 0-31 are ecx
        /// Bits 32-63 are edx
        pub struct CpuFeatures(pub u64);
        impl Default for CpuFeatures {
            fn default() -> Self {
                let features = cpuid(CpuidQuery::GetCpuFeatures);
                Self(((features.edx as u64) << 32) | (features.ecx as u64))
            }
        }
        impl CpuFeatures {
            /// Checks if the feature is available
            ///
            /// `This method is auto-generated`
            #[inline]
            #[allow(dead_code)]
            #[allow(unused_attributes)]
            pub const fn has_apic(&self) -> bool {
                self.0 & (1 << ((CpuFeatureEdx::APIC as u64) << 32).trailing_zeros())
                    != 0
            }
        }
    }
    pub mod interrupts {
        use core::arch::asm;
        /// x86/x86_64-only.
        ///
        /// # Safety
        /// Enable interrupt, which could lead to undefined behavior
        #[inline(always)]
        pub unsafe fn enable() {
            unsafe { asm!("sti", options(preserves_flags, nostack)) };
        }
        /// x86/x86_64-only.
        ///
        /// # Safety
        /// Disable interrupts, which could lead into undefined behavior.
        #[inline(always)]
        pub unsafe fn disable() {
            unsafe { asm!("cli", options(preserves_flags, nostack)) };
        }
        /// x86/x86_64-only.
        ///
        /// # Safety
        /// Halts the CPU until the next external interrupt. Could stop the system
        /// entirely if interrupts are disabled
        #[inline(always)]
        pub unsafe fn hlt() {
            unsafe { asm!("hlt", options(nomem, nostack)) };
        }
    }
    pub mod macros {
        pub(crate) use cpu_feature;
    }
    pub mod port {
        use common::enums::Port;
        use core::arch::asm;
        use extend::ext;
        #[allow(non_camel_case_types)]
        pub trait PortExt {
            #[allow(
                patterns_in_fns_without_body,
                clippy::inline_fn_without_body,
                unused_attributes
            )]
            #[inline(always)]
            unsafe fn outb(&mut self, val: u8);
            #[allow(
                patterns_in_fns_without_body,
                clippy::inline_fn_without_body,
                unused_attributes
            )]
            #[inline(always)]
            unsafe fn outw(&mut self, val: u16);
            #[allow(
                patterns_in_fns_without_body,
                clippy::inline_fn_without_body,
                unused_attributes
            )]
            #[inline(always)]
            unsafe fn outl(&mut self, val: u32);
            #[allow(
                patterns_in_fns_without_body,
                clippy::inline_fn_without_body,
                unused_attributes
            )]
            /// IN instructions
            #[inline(always)]
            unsafe fn inb(&self) -> u8;
            #[allow(
                patterns_in_fns_without_body,
                clippy::inline_fn_without_body,
                unused_attributes
            )]
            #[inline(always)]
            unsafe fn inw(&self) -> u16;
            #[allow(
                patterns_in_fns_without_body,
                clippy::inline_fn_without_body,
                unused_attributes
            )]
            #[inline(always)]
            unsafe fn inl(&self) -> u32;
            #[allow(
                patterns_in_fns_without_body,
                clippy::inline_fn_without_body,
                unused_attributes
            )]
            #[inline(always)]
            unsafe fn iowait();
        }
        impl PortExt for Port {
            #[inline(always)]
            unsafe fn outb(&mut self, val: u8) {
                unsafe {
                    asm!(
                        "out dx, al", in ("dx") * self as u16, in ("al") val,
                        options(preserves_flags, nostack)
                    );
                }
            }
            #[inline(always)]
            unsafe fn outw(&mut self, val: u16) {
                unsafe {
                    asm!(
                        "out dx, ax", in ("dx") * self as u16, in ("ax") val,
                        options(preserves_flags, nostack)
                    );
                }
            }
            #[inline(always)]
            unsafe fn outl(&mut self, val: u32) {
                unsafe {
                    asm!(
                        "out dx, eax", in ("dx") * self as u16, in ("eax") val,
                        options(preserves_flags, nostack)
                    );
                }
            }
            /// IN instructions
            #[inline(always)]
            unsafe fn inb(&self) -> u8 {
                let mut val: u8;
                unsafe {
                    asm!(
                        "in al, dx", in ("dx") * self as u16, out("al") val,
                        options(preserves_flags, nostack)
                    );
                }
                val
            }
            #[inline(always)]
            unsafe fn inw(&self) -> u16 {
                let mut val: u16;
                unsafe {
                    asm!(
                        "in ax, dx", in ("dx") * self as u16, out("ax") val,
                        options(preserves_flags, nostack)
                    );
                }
                val
            }
            #[inline(always)]
            unsafe fn inl(&self) -> u32 {
                let mut val: u32;
                unsafe {
                    asm!(
                        "in eax, dx", in ("dx") * self as u16, out("eax") val,
                        options(preserves_flags, nostack)
                    );
                }
                val
            }
            #[inline(always)]
            unsafe fn iowait() {
                unsafe {
                    Port::IOWait.outb(0);
                };
            }
        }
    }
    pub mod tables {
        use core::arch::asm;
        use crate::structures::{
            global_descriptor_table::{
                GlobalDescriptorTableLong, GlobalDescriptorTableRegister,
            },
            interrupt_descriptor_table::InterruptDescriptorTableRegister,
            segments::SegmentSelector,
        };
        use core::mem::MaybeUninit;
        /// Load the global descriptor table.
        ///
        /// # Safety
        /// This function overrides the current GDT if defined.
        pub unsafe fn lgdt(gdtr: &GlobalDescriptorTableRegister) {
            unsafe {
                asm!(
                    "lgdt [{0}]", in (reg) gdtr, options(readonly, preserves_flags,
                    nostack)
                );
            }
        }
        /// Load the interrupt descriptor table.
        ///
        /// # Safety
        /// This function overrides the current IDT if defined.
        pub unsafe fn lidt(idtr: &InterruptDescriptorTableRegister) {
            unsafe {
                asm!(
                    "lidt [{0}]", in (reg) idtr, options(readonly, preserves_flags,
                    nostack)
                );
            }
        }
        /// Store the content of the global descriptor table
        ///
        /// # Safety
        /// There is no way to check if the register has valid data in it, or it is
        /// not initialized
        pub unsafe fn sgdt() -> &'static GlobalDescriptorTableLong {
            let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> = MaybeUninit::uninit();
            unsafe {
                asm!(
                    "sgdt [{0}]", in (reg) gdt_register.as_mut_ptr(),
                    options(preserves_flags, nostack)
                );
                &mut *(gdt_register.assume_init().base
                    as *mut &GlobalDescriptorTableLong)
            }
        }
        /// Load the task register
        ///
        /// # Safety
        /// This function does not check if the segment selector points into a
        /// valid SegmentSelector
        pub unsafe fn ltr(selector: SegmentSelector) {
            unsafe { asm!("ltr {0:x}", in (reg) u16::from(selector)) }
        }
    }
    pub mod tlb {
        use common::address_types::{Address, VirtualAddress};
        use core::arch::asm;
        use crate::registers::cr3;
        pub fn flash_address(address: VirtualAddress) {
            unsafe {
                asm!(
                    "invlpg [{0:r}]", in (reg) address.as_usize(),
                    options(preserves_flags, nostack)
                )
            }
        }
        pub fn flash_all() {
            let cr3 = cr3::read();
            let _ = cr3::write(cr3);
        }
    }
    pub use tables::*;
}
pub mod memory_map {
    use common::{
        constants::{INIT_AREA_SIZE_BYTES, KiB, MiB},
        enums::MemoryRegionType,
    };
    use core::{
        fmt::{self, Display, Formatter},
        ops::Deref, ptr::NonNull,
    };
    #[repr(C)]
    pub struct MemoryRegionExtended {
        base_address: u64,
        length: u64,
        region_type: MemoryRegionType,
        extended_attributes: u32,
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for MemoryRegionExtended {}
    #[automatically_derived]
    impl ::core::clone::Clone for MemoryRegionExtended {
        #[inline]
        fn clone(&self) -> MemoryRegionExtended {
            let _: ::core::clone::AssertParamIsClone<u64>;
            let _: ::core::clone::AssertParamIsClone<MemoryRegionType>;
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MemoryRegionExtended {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "MemoryRegionExtended",
                "base_address",
                &self.base_address,
                "length",
                &self.length,
                "region_type",
                &self.region_type,
                "extended_attributes",
                &&self.extended_attributes,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for MemoryRegionExtended {}
    #[repr(C)]
    pub struct MemoryRegion {
        pub base_address: u64,
        pub length: u64,
        pub region_type: MemoryRegionType,
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for MemoryRegion {}
    #[automatically_derived]
    impl ::core::clone::Clone for MemoryRegion {
        #[inline]
        fn clone(&self) -> MemoryRegion {
            let _: ::core::clone::AssertParamIsClone<u64>;
            let _: ::core::clone::AssertParamIsClone<MemoryRegionType>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MemoryRegion {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "MemoryRegion",
                "base_address",
                &self.base_address,
                "length",
                &self.length,
                "region_type",
                &&self.region_type,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for MemoryRegion {}
    impl From<&MemoryRegionExtended> for MemoryRegion {
        fn from(value: &MemoryRegionExtended) -> Self {
            MemoryRegion {
                base_address: value.base_address,
                length: value.length,
                region_type: value.region_type,
            }
        }
    }
    pub enum MemoryMapError {
        Empty,
        Overflow,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MemoryMapError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    MemoryMapError::Empty => "Empty",
                    MemoryMapError::Overflow => "Overflow",
                },
            )
        }
    }
    pub struct MemoryMap {
        pub regions: NonNull<[MemoryRegion]>,
        pub capacity: usize,
    }
    impl Deref for MemoryMap {
        type Target = [MemoryRegion];
        fn deref(&self) -> &Self::Target {
            unsafe { self.regions.as_ref() }
        }
    }
    impl Display for MemoryMap {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let mut usable = 0u64;
            let mut reserved = 0u64;
            for entry in self.iter() {
                let size_mib = entry.length / MiB as u64;
                let size_kib = (entry.length % MiB as u64) / KiB as u64;
                f.write_fmt(
                    format_args!(
                        "[0x{0:0>9x} - 0x{1:0>9x}]: type: {2:?}",
                        entry.base_address,
                        entry.base_address + entry.length,
                        entry.region_type,
                    ),
                )?;
                match entry.region_type {
                    MemoryRegionType::Usable
                    | MemoryRegionType::Reserved
                    | MemoryRegionType::Filler => {
                        if entry.region_type == MemoryRegionType::Usable {
                            usable += entry.length;
                        }
                        if entry.region_type == MemoryRegionType::Reserved {
                            reserved += entry.length;
                        }
                        f.write_fmt(
                            format_args!(
                                " (Size: {0:>4} MiB {1:>4} KiB)\n",
                                size_mib,
                                size_kib,
                            ),
                        )?;
                    }
                    _ => f.write_fmt(format_args!("\n"))?,
                }
            }
            f.write_fmt(format_args!("\n"))?;
            f.write_fmt(
                format_args!(
                    "Total Usable Memory:   {0:>5} MiB {1:>4} KiB\n",
                    usable / MiB as u64,
                    (usable % MiB as u64) / KiB as u64,
                ),
            )?;
            f.write_fmt(
                format_args!(
                    "Total Reserved Memory: {0:>5} MiB {1:>4} KiB\n",
                    reserved / MiB as u64,
                    (reserved % MiB as u64) / KiB as u64,
                ),
            )
        }
    }
    impl MemoryMap {
        pub fn parse_map(
            raw: &mut [MemoryRegionExtended],
            buf: &mut [MemoryRegion],
        ) -> Result<MemoryMap, MemoryMapError> {
            let mut position = 0;
            let mut push = |region: MemoryRegion| -> Result<(), MemoryMapError> {
                if position >= buf.len() {
                    return Err(MemoryMapError::Overflow);
                }
                buf[position] = region;
                position += 1;
                Ok(())
            };
            let first_usable = raw
                .iter_mut()
                .find(|f| f.region_type == MemoryRegionType::Usable)
                .ok_or(MemoryMapError::Empty)?;
            if !(first_usable.length > INIT_AREA_SIZE_BYTES) {
                ::core::panicking::panic(
                    "assertion failed: first_usable.length > INIT_AREA_SIZE_BYTES",
                )
            }
            let init_entry = MemoryRegion {
                base_address: first_usable.base_address,
                length: INIT_AREA_SIZE_BYTES,
                region_type: MemoryRegionType::UserEnterd,
            };
            first_usable.base_address += INIT_AREA_SIZE_BYTES;
            first_usable.length -= INIT_AREA_SIZE_BYTES;
            push(init_entry)?;
            for (left, right) in raw.iter().map_windows(|[a, b]| (*a, *b)) {
                let filler = filler_entry(left, right);
                push(left.into())?;
                if let Some(f) = filler {
                    push(f)?;
                }
            }
            let last = raw.last().ok_or(MemoryMapError::Empty)?;
            push(last.into())?;
            let capacity = buf.len();
            let modified = unsafe {
                core::slice::from_raw_parts_mut(buf.as_mut_ptr(), position)
            };
            Ok(MemoryMap {
                regions: NonNull::from_mut(modified),
                capacity,
            })
        }
    }
    /// Return a filler entry if there is a gap between A and B
    #[inline]
    fn filler_entry(
        left: &MemoryRegionExtended,
        right: &MemoryRegionExtended,
    ) -> Option<MemoryRegion> {
        if !(left.base_address < right.base_address) {
            ::core::panicking::panic(
                "assertion failed: left.base_address < right.base_address",
            )
        }
        (left.base_address + left.length < right.base_address)
            .then(|| {
                let filler_base = left.base_address + left.length;
                let length = right.base_address - filler_base;
                MemoryRegion {
                    base_address: filler_base,
                    length,
                    region_type: MemoryRegionType::Filler,
                }
            })
    }
}
pub mod pic8259 {
    /// The code in this module is inspired from osdev
    /// 8259_PIC guide.
    use crate::instructions::port::PortExt;
    use common::enums::{
        CascadedPicInterruptLine, PicCommandCode, PicInterruptLine,
        PicInterruptVectorOffset, PicMode, Port,
    };
    pub static mut PIC: CascadedPIC = CascadedPIC::default();
    struct ProgrammableInterruptController {
        command: Port,
        data: Port,
        interrupt_offset: PicInterruptVectorOffset,
    }
    impl ProgrammableInterruptController {
        const fn default_master() -> Self {
            Self {
                command: Port::MasterPicCmd,
                data: Port::MasterPicData,
                interrupt_offset: PicInterruptVectorOffset::Master,
            }
        }
        const fn default_slave() -> Self {
            Self {
                command: Port::SlavePicCmd,
                data: Port::SlavePicData,
                interrupt_offset: PicInterruptVectorOffset::Slave,
            }
        }
        fn disable_irq(&mut self, irq: PicInterruptLine) {
            unsafe {
                let prev_mask = self.data.inb();
                let new_mask = prev_mask | irq as u8;
                self.data.outb(new_mask);
            }
        }
        fn enable_irq(&mut self, irq: PicInterruptLine) {
            unsafe {
                let prev_mask = self.data.inb();
                let new_mask = prev_mask & !(irq as u8);
                self.data.outb(new_mask);
            }
        }
        fn enable(&mut self) {
            unsafe {
                self.data.outb(0);
            }
        }
        fn end_of_interrupt(&mut self) {
            unsafe {
                self.command.outb(PicCommandCode::EndOfInterrupt as u8);
            }
        }
    }
    pub struct CascadedPIC {
        master: ProgrammableInterruptController,
        slave: ProgrammableInterruptController,
    }
    impl CascadedPIC {
        pub const fn default() -> Self {
            Self {
                master: ProgrammableInterruptController::default_master(),
                slave: ProgrammableInterruptController::default_slave(),
            }
        }
        pub fn init(uninit: &'static mut Self) {
            unsafe {
                uninit
                    .master
                    .command
                    .outb(
                        PicCommandCode::Initialize as u8
                            | PicCommandCode::CascadeMode as u8,
                    );
                Port::iowait();
                uninit
                    .slave
                    .command
                    .outb(
                        PicCommandCode::Initialize as u8
                            | PicCommandCode::CascadeMode as u8,
                    );
                Port::iowait();
                uninit.master.data.outb(uninit.master.interrupt_offset as u8);
                Port::iowait();
                uninit.slave.data.outb(uninit.slave.interrupt_offset as u8);
                Port::iowait();
                uninit.master.data.outb(PicInterruptLine::Irq2 as u8);
                Port::iowait();
                uninit.slave.data.outb(PicInterruptLine::Irq1 as u8);
                Port::iowait();
                uninit.master.data.outb(PicMode::Mode8086 as u8);
                Port::iowait();
                uninit.slave.data.outb(PicMode::Mode8086 as u8);
                Port::iowait();
                uninit.master.enable();
                uninit.slave.enable();
            }
        }
        pub fn disable_irq(&mut self, irq: CascadedPicInterruptLine) {
            unsafe {
                if irq as u16 > PicInterruptLine::Irq7 as u16 {
                    let irq: PicInterruptLine = core::mem::transmute(
                        ((irq as u16) >> u8::BITS) as u8,
                    );
                    self.slave.disable_irq(irq);
                } else {
                    let irq: PicInterruptLine = core::mem::transmute(irq as u8);
                    self.master.disable_irq(irq);
                }
            }
        }
        pub fn enable_irq(&mut self, irq: CascadedPicInterruptLine) {
            unsafe {
                if irq as u16 >= CascadedPicInterruptLine::Irq8 as u16 {
                    let irq: PicInterruptLine = core::mem::transmute(
                        ((irq as u16) >> u8::BITS) as u8,
                    );
                    self.slave.enable_irq(irq);
                } else {
                    let irq: PicInterruptLine = core::mem::transmute(irq as u8);
                    self.master.enable_irq(irq);
                }
            }
        }
        pub fn end_of_interrupt(&mut self, irq: CascadedPicInterruptLine) {
            if irq as u16 >= CascadedPicInterruptLine::Irq8 as u16 {
                self.slave.end_of_interrupt();
            }
            self.master.end_of_interrupt();
        }
    }
}
pub mod registers {
    pub mod control {
        use crate::registers::macros::impl_reg_read_write_u64;
        pub mod cr3 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov cr3, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, cr3", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod cr2 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov cr2, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, cr2", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
    }
    pub mod general_purpose {
        use crate::registers::macros::{
            impl_reg_read_write_u8, impl_reg_read_write_u16, impl_reg_read_write_u32,
            impl_reg_read_write_u64,
        };
        pub mod rax {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rax, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rax", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod eax {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, eax", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov eax, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod ax {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, ax", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ax, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod ah {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, ah", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ah, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod al {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, al", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov al, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod rbx {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rbx, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rbx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod ebx {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, ebx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ebx, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod bx {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, bx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov bx, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod bh {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, bh", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov bh, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod bl {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, bl", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov bl, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod rcx {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rcx, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rcx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod ecx {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, ecx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ecx, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod cx {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, cx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov cx, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod ch {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, ch", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ch, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod cl {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, cl", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov cl, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod rdx {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rdx, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rdx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod edx {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, edx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov edx, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod dx {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, dx", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov dx, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod dh {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, dh", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov dh, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod dl {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u8 {
                let val: u8;
                unsafe {
                    asm!(
                        "mov {0}, dl", out(reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u8) -> u8 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov dl, {0}", in (reg_byte) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod rsi {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rsi, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rsi", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod esi {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, esi", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov esi, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod si {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, si", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov si, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod rdi {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rdi, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rdi", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod edi {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, edi", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov edi, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod di {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, di", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov di, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod rsp {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rsp, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rsp", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod esp {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, esp", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov esp, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod sp {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, sp", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov sp, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod rbp {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov rbp, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, rbp", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod ebp {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, ebp", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ebp, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod bp {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, bp", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov bp, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r8 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r8, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r8", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r8d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r8d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r8d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r8w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r8w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r8w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r9 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r9, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r9", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r9d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r9d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r9d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r9w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r9w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r9w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r10 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r10, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r10", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r10d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r10d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r10d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r10w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r10w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r10w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r11 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r11, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r11", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r11d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r11d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r11d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r11w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r11w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r11w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r12 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r12, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r12", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r12d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r12d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r12d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r12w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r12w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r12w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r13 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r13, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r13", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r13d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r13d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r13d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r13w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r13w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r13w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r14 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r14, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r14", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r14d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r14d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r14d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r14w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r14w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r14w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r15 {
            use core::arch::asm;
            #[inline(always)]
            pub fn write(val: u64) -> u64 {
                let prev = read();
                if val != prev {
                    unsafe {
                        asm!(
                            "mov r15, {0}", in (reg) val, options(nomem, preserves_flags,
                            nostack)
                        )
                    }
                }
                prev
            }
            #[inline(always)]
            pub fn read() -> u64 {
                unsafe {
                    let val: u64;
                    asm!(
                        "mov {0}, r15", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                    val
                }
            }
        }
        pub mod r15d {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u32 {
                let val: u32;
                unsafe {
                    asm!(
                        "mov {0:e}, r15d", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u32) -> u32 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r15d, {0:e}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod r15w {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, r15w", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov r15w, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
    }
    mod macros {
        pub(crate) use impl_reg_read_write_u8;
        pub(crate) use impl_reg_read_write_u16;
        pub(crate) use impl_reg_read_write_u32;
        pub(crate) use impl_reg_read_write_u64;
    }
    pub mod model_specific {
        use common::enums::MSR;
        use core::arch::asm;
        /// Read from the given model specific register
        pub fn rdmsr(msr: MSR) -> u64 {
            let low: u32;
            let high: u32;
            unsafe {
                asm!("rdmsr", in ("ecx") msr as u32, out("eax") low, out("edx") high);
            }
            ((high as u64) << 32) | (low as u64)
        }
        /// Write `value` to the given model specific register
        ///
        /// # Safety
        /// This function writes arbitrary value to the register, which could lead
        /// into undefined behavior
        pub unsafe fn wrmsr(msr: MSR, value: u64) {
            let low = value as u32;
            let high = (value >> 32) as u32;
            unsafe {
                asm!(
                    "wrmsr", in ("ecx") msr as u32, in ("eax") low, in ("edx") high,
                    options(preserves_flags, nostack)
                );
            }
        }
    }
    pub mod rflags {
        use common::enums::ProtectionLevel;
        use core::arch::asm;
        use macros::bitfields;
        #[repr(transparent)]
        pub struct Rflags(u64);
        #[automatically_derived]
        impl ::core::marker::Copy for Rflags {}
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for Rflags {}
        #[automatically_derived]
        impl ::core::clone::Clone for Rflags {
            #[inline]
            fn clone(&self) -> Rflags {
                let _: ::core::clone::AssertParamIsClone<u64>;
                *self
            }
        }
        impl Rflags {
            #[inline]
            pub const fn new() -> Self {
                Self(0)
            }
            #[inline]
            #[track_caller]
            fn is_carry(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 0usize;
                    let bits = (val & mask) >> 0usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_carry(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_carry: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 0usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 0usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn carry(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::carry: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 0usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_reserved1(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 1usize;
                    let bits = (val & mask) >> 1usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn is_parity(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 2usize;
                    let bits = (val & mask) >> 2usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_parity(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_parity: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 2usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 2usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn parity(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::parity: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 2usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_reserved2(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 3usize;
                    let bits = (val & mask) >> 3usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn is_auxiliary(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 4usize;
                    let bits = (val & mask) >> 4usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_auxiliary(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_auxiliary: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 4usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 4usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn auxiliary(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::auxiliary: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 4usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_reserved3(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 5usize;
                    let bits = (val & mask) >> 5usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn is_zero(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 6usize;
                    let bits = (val & mask) >> 6usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_zero(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_zero: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 6usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 6usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn zero(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::zero: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 6usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_sign(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 7usize;
                    let bits = (val & mask) >> 7usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_sign(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_sign: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 7usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 7usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn sign(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::sign: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 7usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_tap(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 8usize;
                    let bits = (val & mask) >> 8usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_tap(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_tap: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 8usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 8usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn tap(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::tap: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 8usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_interrupt(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 9usize;
                    let bits = (val & mask) >> 9usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_interrupt(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_interrupt: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 9usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 9usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn interrupt(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::interrupt: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 9usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_direction(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 10usize;
                    let bits = (val & mask) >> 10usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_direction(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_direction: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 10usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 10usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn direction(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::direction: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 10usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_overflow(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 11usize;
                    let bits = (val & mask) >> 11usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_overflow(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_overflow: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 11usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 11usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn overflow(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::overflow: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 11usize;
                self
            }
            #[inline]
            #[track_caller]
            fn get_iopl(&self) -> ProtectionLevel {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 2usize as u32)) << 12usize;
                    let bits = (val & mask) >> 12usize;
                    <ProtectionLevel as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into ProtectionLevel")
                }
            }
            #[inline]
            #[track_caller]
            fn set_iopl(&mut self, v: ProtectionLevel) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u64) <= (3u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_iopl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 2usize as u32)) << 12usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 12usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn iopl(mut self, v: ProtectionLevel) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u64) <= (3u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::iopl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 12usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_nested_task(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 14usize;
                    let bits = (val & mask) >> 14usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_nested_task(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_nested_task: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 14usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 14usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn nested_task(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::nested_task: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 14usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_reserved4(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 15usize;
                    let bits = (val & mask) >> 15usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_reserved4(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_reserved4: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 15usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 15usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn reserved4(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::reserved4: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 15usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_resume(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 16usize;
                    let bits = (val & mask) >> 16usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_resume(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_resume: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 16usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 16usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn resume(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::resume: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 16usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_virtual_8086_mode(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 17usize;
                    let bits = (val & mask) >> 17usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_virtual_8086_mode(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_virtual_8086_mode: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 17usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 17usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn virtual_8086_mode(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::virtual_8086_mode: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 17usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_alignment_check(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 18usize;
                    let bits = (val & mask) >> 18usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_alignment_check(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_alignment_check: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 18usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 18usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn alignment_check(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::alignment_check: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 18usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_virtual_interrupt(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 19usize;
                    let bits = (val & mask) >> 19usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_virtual_interrupt(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_virtual_interrupt: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 19usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 19usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn virtual_interrupt(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::virtual_interrupt: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 19usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_virtual_interrupt_pending(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 20usize;
                    let bits = (val & mask) >> 20usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_virtual_interrupt_pending(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_virtual_interrupt_pending: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 20usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 20usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn virtual_interrupt_pending(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::virtual_interrupt_pending: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 20usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_cpuid_support(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 21usize;
                    let bits = (val & mask) >> 21usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_cpuid_support(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::set_cpuid_support: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 21usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u64) << 21usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn cpuid_support(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u64) <= (1u128 as u64)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Rflags::cpuid_support: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u64) << 21usize;
                self
            }
            #[inline]
            #[track_caller]
            fn get_reserved5(&self) -> u64 {
                unsafe {
                    let addr = self as *const _ as *mut u64;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u64::MAX >> (u64::BITS - 41usize as u32)) << 22usize;
                    let bits = (val & mask) >> 22usize;
                    <u64 as ::core::convert::TryFrom<u64>>::try_from(bits as u64)
                        .expect("Cannot convert bit representation into u64")
                }
            }
        }
        impl const ::core::convert::From<u64> for Rflags {
            fn from(value: u64) -> Self {
                Rflags(value)
            }
        }
        impl const ::core::convert::From<Rflags> for u64 {
            fn from(value: Rflags) -> Self {
                value.0
            }
        }
        impl ::core::fmt::Debug for Rflags {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("Rflags")
                    .field("carry", &self.is_carry())
                    .field("reserved1", &self.is_reserved1())
                    .field("parity", &self.is_parity())
                    .field("reserved2", &self.is_reserved2())
                    .field("auxiliary", &self.is_auxiliary())
                    .field("reserved3", &self.is_reserved3())
                    .field("zero", &self.is_zero())
                    .field("sign", &self.is_sign())
                    .field("tap", &self.is_tap())
                    .field("interrupt", &self.is_interrupt())
                    .field("direction", &self.is_direction())
                    .field("overflow", &self.is_overflow())
                    .field("iopl", &self.get_iopl())
                    .field("nested_task", &self.is_nested_task())
                    .field("reserved4", &self.is_reserved4())
                    .field("resume", &self.is_resume())
                    .field("virtual_8086_mode", &self.is_virtual_8086_mode())
                    .field("alignment_check", &self.is_alignment_check())
                    .field("virtual_interrupt", &self.is_virtual_interrupt())
                    .field(
                        "virtual_interrupt_pending",
                        &self.is_virtual_interrupt_pending(),
                    )
                    .field("cpuid_support", &self.is_cpuid_support())
                    .field("reserved5", &self.get_reserved5())
                    .finish()
            }
        }
        impl Rflags {
            pub fn read() -> Self {
                let r: u64;
                unsafe {
                    asm!("pushfq\npop {0}", out(reg) r, options(nomem, preserves_flags));
                }
                Self(r)
            }
            /// Write the given flags to the cpu flags overriding current flags
            ///
            /// # Safety
            /// Writing custom flags is very risky, and can easily lead into
            /// undefined behavior
            pub unsafe fn write(&mut self, flags: Self) {
                unsafe {
                    asm!(
                        "push {0}\npopfq", in (reg) flags.0, options(nomem,
                        preserves_flags)
                    );
                }
            }
        }
    }
    pub mod segment {
        use crate::registers::macros::impl_reg_read_write_u16;
        pub mod ss {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, ss", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ss, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod ds {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, ds", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov ds, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
        pub mod es {
            use core::arch::asm;
            #[inline(always)]
            pub fn read() -> u16 {
                let val: u16;
                unsafe {
                    asm!(
                        "mov {0:x}, es", out(reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                val
            }
            #[inline(always)]
            pub fn write(val: u16) -> u16 {
                let prev = read();
                unsafe {
                    asm!(
                        "mov es, {0:x}", in (reg) val, options(nomem, preserves_flags,
                        nostack)
                    );
                }
                prev
            }
        }
    }
    pub use control::*;
    pub use general_purpose::*;
    pub use segment::*;
}
pub mod structures {
    pub mod global_descriptor_table {
        use common::enums::{
            ProtectionLevel, Sections, SegmentDescriptorType, SystemSegmentType,
        };
        use macros::bitfields;
        use crate::{instructions, structures::segments::SegmentSelector};
        #[repr(transparent)]
        pub struct AccessByte(u8);
        #[automatically_derived]
        impl ::core::marker::Copy for AccessByte {}
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for AccessByte {}
        #[automatically_derived]
        impl ::core::clone::Clone for AccessByte {
            #[inline]
            fn clone(&self) -> AccessByte {
                let _: ::core::clone::AssertParamIsClone<u8>;
                *self
            }
        }
        impl AccessByte {
            #[inline]
            pub const fn new() -> Self {
                Self(0)
            }
            #[inline]
            #[track_caller]
            fn is_accessed(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 0usize;
                    let bits = (val & mask) >> 0usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn is_readable_writable(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 1usize;
                    let bits = (val & mask) >> 1usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_readable_writable(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::set_readable_writable: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 1usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 1usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn readable_writable(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::readable_writable: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 1usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_direction_conforming(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 2usize;
                    let bits = (val & mask) >> 2usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_direction_conforming(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::set_direction_conforming: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 2usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 2usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn direction_conforming(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::direction_conforming: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 2usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_executable(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 3usize;
                    let bits = (val & mask) >> 3usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_executable(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::set_executable: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 3usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 3usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn executable(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::executable: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 3usize;
                self
            }
            #[inline]
            #[track_caller]
            fn get_segment_type(&self) -> SegmentDescriptorType {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 4usize;
                    let bits = (val & mask) >> 4usize;
                    <SegmentDescriptorType as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect(
                            "Cannot convert bit representation into SegmentDescriptorType",
                        )
                }
            }
            #[inline]
            #[track_caller]
            fn set_segment_type(&mut self, v: SegmentDescriptorType) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (SegmentDescriptorType) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::set_segment_type: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 4usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 4usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn segment_type(mut self, v: SegmentDescriptorType) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (SegmentDescriptorType) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::segment_type: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 4usize;
                self
            }
            #[inline]
            #[track_caller]
            fn get_dpl(&self) -> ProtectionLevel {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 2usize as u32)) << 5usize;
                    let bits = (val & mask) >> 5usize;
                    <ProtectionLevel as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into ProtectionLevel")
                }
            }
            #[inline]
            #[track_caller]
            fn set_dpl(&mut self, v: ProtectionLevel) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u8) <= (3u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::set_dpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 2usize as u32)) << 5usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 5usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn dpl(mut self, v: ProtectionLevel) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u8) <= (3u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::dpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 5usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_present(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
                    let bits = (val & mask) >> 7usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_present(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::set_present: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 7usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn present(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "AccessByte::present: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 7usize;
                self
            }
        }
        impl const ::core::convert::From<u8> for AccessByte {
            fn from(value: u8) -> Self {
                AccessByte(value)
            }
        }
        impl const ::core::convert::From<AccessByte> for u8 {
            fn from(value: AccessByte) -> Self {
                value.0
            }
        }
        impl ::core::fmt::Debug for AccessByte {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("AccessByte")
                    .field("accessed", &self.is_accessed())
                    .field("readable_writable", &self.is_readable_writable())
                    .field("direction_conforming", &self.is_direction_conforming())
                    .field("executable", &self.is_executable())
                    .field("segment_type", &self.get_segment_type())
                    .field("dpl", &self.get_dpl())
                    .field("present", &self.is_present())
                    .finish()
            }
        }
        /// Low 4 bits limit high 4 bits flags
        #[repr(transparent)]
        struct LimitFlags(u8);
        #[automatically_derived]
        impl ::core::marker::Copy for LimitFlags {}
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for LimitFlags {}
        #[automatically_derived]
        impl ::core::clone::Clone for LimitFlags {
            #[inline]
            fn clone(&self) -> LimitFlags {
                let _: ::core::clone::AssertParamIsClone<u8>;
                *self
            }
        }
        impl LimitFlags {
            #[inline]
            pub const fn new() -> Self {
                Self(0)
            }
            #[inline]
            #[track_caller]
            fn get_limit_high(&self) -> u8 {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 4usize as u32)) << 0usize;
                    let bits = (val & mask) >> 0usize;
                    <u8 as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into u8")
                }
            }
            #[inline]
            #[track_caller]
            fn set_limit_high(&mut self, v: u8) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (u8) into u8");
                if true {
                    if !((v as u8) <= (15u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::set_limit_high: value out of range: must fit in 4 bits (max 0xf)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 4usize as u32)) << 0usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 0usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn limit_high(mut self, v: u8) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (u8) into u8");
                if true {
                    if !((v as u8) <= (15u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::limit_high: value out of range: must fit in 4 bits (max 0xf)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 0usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_reserved(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 4usize;
                    let bits = (val & mask) >> 4usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn is_long(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 5usize;
                    let bits = (val & mask) >> 5usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_long(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::set_long: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 5usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 5usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn long(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::long: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 5usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_protected(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 6usize;
                    let bits = (val & mask) >> 6usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_protected(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::set_protected: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 6usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 6usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn protected(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::protected: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 6usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_granularity(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
                    let bits = (val & mask) >> 7usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_granularity(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::set_granularity: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 7usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn granularity(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "LimitFlags::granularity: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 7usize;
                self
            }
        }
        impl const ::core::convert::From<u8> for LimitFlags {
            fn from(value: u8) -> Self {
                LimitFlags(value)
            }
        }
        impl const ::core::convert::From<LimitFlags> for u8 {
            fn from(value: LimitFlags) -> Self {
                value.0
            }
        }
        impl ::core::fmt::Debug for LimitFlags {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("LimitFlags")
                    .field("limit_high", &self.get_limit_high())
                    .field("reserved", &self.is_reserved())
                    .field("long", &self.is_long())
                    .field("protected", &self.is_protected())
                    .field("granularity", &self.is_granularity())
                    .finish()
            }
        }
        #[repr(C, packed)]
        struct GlobalDescriptorTableEntry32 {
            limit_low: u16,
            base_low: u16,
            base_mid: u8,
            access_byte: AccessByte,
            limit_flags: LimitFlags,
            base_high: u8,
        }
        impl const Default for GlobalDescriptorTableEntry32 {
            fn default() -> Self {
                GlobalDescriptorTableEntry32 {
                    limit_low: 0,
                    base_low: 0,
                    base_mid: 0,
                    access_byte: AccessByte::new(),
                    limit_flags: LimitFlags::new(),
                    base_high: 0,
                }
            }
        }
        impl GlobalDescriptorTableEntry32 {
            /// Create a new entry
            ///
            /// # Parameters
            ///
            /// - `base`: The base address of the segment
            /// - `limit`: The size of the segment
            /// - `access_byte`: The type and access privileges of the entry
            /// - `flags`: Configuration flags of the entry
            pub const fn new(
                base: u32,
                limit: u32,
                access_byte: AccessByte,
                flags: LimitFlags,
            ) -> GlobalDescriptorTableEntry32 {
                let base_low = (base & 0xffff) as u16;
                let base_mid = ((base >> 0x10) & 0xff) as u8;
                let base_high = ((base >> 0x18) & 0xff) as u8;
                let limit_low = (limit & 0xffff) as u16;
                let limit_high = ((limit >> 0x10) & 0xf) as u8;
                let limit_flags = flags.0 | limit_high;
                GlobalDescriptorTableEntry32 {
                    limit_low,
                    base_low,
                    base_mid,
                    access_byte,
                    limit_flags: LimitFlags(limit_flags),
                    base_high,
                }
            }
        }
        #[repr(C, packed)]
        pub struct GlobalDescriptorTableRegister {
            pub limit: u16,
            pub base: usize,
        }
        #[repr(transparent)]
        pub struct SystemAccessByte(u8);
        #[automatically_derived]
        impl ::core::marker::Copy for SystemAccessByte {}
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for SystemAccessByte {}
        #[automatically_derived]
        impl ::core::clone::Clone for SystemAccessByte {
            #[inline]
            fn clone(&self) -> SystemAccessByte {
                let _: ::core::clone::AssertParamIsClone<u8>;
                *self
            }
        }
        impl SystemAccessByte {
            #[inline]
            pub const fn new() -> Self {
                Self(0)
            }
            #[inline]
            #[track_caller]
            fn get_segment_type(&self) -> SystemSegmentType {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 4usize as u32)) << 0usize;
                    let bits = (val & mask) >> 0usize;
                    <SystemSegmentType as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect(
                            "Cannot convert bit representation into SystemSegmentType",
                        )
                }
            }
            #[inline]
            #[track_caller]
            fn set_segment_type(&mut self, v: SystemSegmentType) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (SystemSegmentType) into u8");
                if true {
                    if !((v as u8) <= (15u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SystemAccessByte::set_segment_type: value out of range: must fit in 4 bits (max 0xf)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 4usize as u32)) << 0usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 0usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn segment_type(mut self, v: SystemSegmentType) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (SystemSegmentType) into u8");
                if true {
                    if !((v as u8) <= (15u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SystemAccessByte::segment_type: value out of range: must fit in 4 bits (max 0xf)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 0usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_zero(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 4usize;
                    let bits = (val & mask) >> 4usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn clear_zero(&mut self) {
                let v = 0usize as u8;
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SystemAccessByte::clear_zero: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 4usize;
                    let cleared = val & !mask;
                    let new = cleared | ((0usize as u8) << 4usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            fn get_dpl(&self) -> ProtectionLevel {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 2usize as u32)) << 5usize;
                    let bits = (val & mask) >> 5usize;
                    <ProtectionLevel as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into ProtectionLevel")
                }
            }
            #[inline]
            #[track_caller]
            fn set_dpl(&mut self, v: ProtectionLevel) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u8) <= (3u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SystemAccessByte::set_dpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 2usize as u32)) << 5usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 5usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn dpl(mut self, v: ProtectionLevel) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u8) <= (3u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SystemAccessByte::dpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 5usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_present(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
                    let bits = (val & mask) >> 7usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_present(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SystemAccessByte::set_present: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u8;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u8::MAX >> (u8::BITS - 1usize as u32)) << 7usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u8) << 7usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn present(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u8) <= (1u128 as u8)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SystemAccessByte::present: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u8) << 7usize;
                self
            }
        }
        impl const ::core::convert::From<u8> for SystemAccessByte {
            fn from(value: u8) -> Self {
                SystemAccessByte(value)
            }
        }
        impl const ::core::convert::From<SystemAccessByte> for u8 {
            fn from(value: SystemAccessByte) -> Self {
                value.0
            }
        }
        impl ::core::fmt::Debug for SystemAccessByte {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("SystemAccessByte")
                    .field("segment_type", &self.get_segment_type())
                    .field("zero", &self.is_zero())
                    .field("dpl", &self.get_dpl())
                    .field("present", &self.is_present())
                    .finish()
            }
        }
        #[repr(C, packed)]
        pub struct SystemSegmentDescriptor64 {
            limit_low: u16,
            base_low: u16,
            base_mid: u8,
            access_byte: SystemAccessByte,
            limit_flags: LimitFlags,
            base_high: u8,
            base_extra: u32,
            _reserved: u32,
        }
        impl const Default for SystemSegmentDescriptor64 {
            fn default() -> Self {
                SystemSegmentDescriptor64 {
                    limit_low: 0,
                    base_low: 0,
                    base_mid: 0,
                    access_byte: SystemAccessByte::new(),
                    limit_flags: LimitFlags::new(),
                    base_high: 0,
                    base_extra: 0,
                    _reserved: 0,
                }
            }
        }
        impl SystemSegmentDescriptor64 {
            /// Construct a new system segment
            ///
            /// # Parameters
            ///
            /// - `base`: The base address of the segment
            /// - `limit`: The limit value of the segment, for each segment this
            ///   may mean something different.
            /// - `segment_type`: The type of the constructed segment
            pub const fn new(
                base: u64,
                limit: u32,
                segment_type: SystemSegmentType,
            ) -> Self {
                let base_low = (base & 0xffff) as u16;
                let base_mid = ((base >> 16) & 0xff) as u8;
                let base_high = ((base >> 24) & 0xff) as u8;
                let limit_low = (limit & 0xffff) as u16;
                let limit_high = ((limit >> 16) & 0xf) as u8;
                let base_extra = (base >> 32) as u32;
                let access_byte = SystemAccessByte::new()
                    .present(true)
                    .dpl(ProtectionLevel::Ring0)
                    .segment_type(segment_type);
                Self {
                    limit_low,
                    base_low,
                    base_mid,
                    access_byte,
                    limit_flags: LimitFlags(limit_high),
                    base_high,
                    base_extra,
                    _reserved: 0,
                }
            }
        }
        /// Initial temporary GDT
        #[repr(C, packed)]
        pub struct GlobalDescriptorTableProtected {
            null: GlobalDescriptorTableEntry32,
            code: GlobalDescriptorTableEntry32,
            data: GlobalDescriptorTableEntry32,
        }
        impl GlobalDescriptorTableProtected {
            /// Creates default global descriptor table for
            /// protected mode
            pub const fn default() -> Self {
                Self {
                    null: GlobalDescriptorTableEntry32::default(),
                    code: GlobalDescriptorTableEntry32::new(
                        0,
                        0xfffff,
                        AccessByte::new()
                            .present(true)
                            .dpl(ProtectionLevel::Ring0)
                            .segment_type(SegmentDescriptorType::CodeOrData)
                            .executable(true)
                            .readable_writable(true),
                        LimitFlags::new().granularity(true).protected(true),
                    ),
                    data: GlobalDescriptorTableEntry32::new(
                        0,
                        0xfffff,
                        AccessByte::new()
                            .present(true)
                            .dpl(ProtectionLevel::Ring0)
                            .segment_type(SegmentDescriptorType::CodeOrData)
                            .readable_writable(true),
                        LimitFlags::new().granularity(true).protected(true),
                    ),
                }
            }
            /// Load the GDT with the `lgdt` instruction
            ///
            /// # Safety
            /// This function doesn't check if a GDT already exists, and just
            /// overrides it.
            pub unsafe fn load(&'static self) {
                let gdtr = {
                    GlobalDescriptorTableRegister {
                        limit: (size_of::<Self>() - 1) as u16,
                        base: self as *const _ as usize,
                    }
                };
                unsafe {
                    instructions::lgdt(&gdtr);
                }
            }
        }
        /// kernel GDT
        #[repr(C, packed)]
        pub struct GlobalDescriptorTableLong {
            null: GlobalDescriptorTableEntry32,
            kernel_code: GlobalDescriptorTableEntry32,
            kernel_data: GlobalDescriptorTableEntry32,
            user_code: GlobalDescriptorTableEntry32,
            user_data: GlobalDescriptorTableEntry32,
            tss: SystemSegmentDescriptor64,
        }
        impl GlobalDescriptorTableLong {
            /// Creates default global descriptor table for long
            /// mode
            pub const fn default() -> Self {
                Self {
                    null: GlobalDescriptorTableEntry32::default(),
                    kernel_code: GlobalDescriptorTableEntry32::new(
                        0,
                        0,
                        AccessByte::new()
                            .segment_type(SegmentDescriptorType::CodeOrData)
                            .present(true)
                            .dpl(ProtectionLevel::Ring0)
                            .readable_writable(true)
                            .executable(true),
                        LimitFlags::new().long(true),
                    ),
                    kernel_data: GlobalDescriptorTableEntry32::new(
                        0,
                        0,
                        AccessByte::new()
                            .segment_type(SegmentDescriptorType::CodeOrData)
                            .present(true)
                            .dpl(ProtectionLevel::Ring0)
                            .readable_writable(true),
                        LimitFlags::new(),
                    ),
                    user_code: GlobalDescriptorTableEntry32::new(
                        0,
                        0,
                        AccessByte::new()
                            .segment_type(SegmentDescriptorType::CodeOrData)
                            .present(true)
                            .dpl(ProtectionLevel::Ring3)
                            .readable_writable(true)
                            .executable(true),
                        LimitFlags::new().long(true),
                    ),
                    user_data: GlobalDescriptorTableEntry32::new(
                        0,
                        0,
                        AccessByte::new()
                            .segment_type(SegmentDescriptorType::CodeOrData)
                            .present(true)
                            .dpl(ProtectionLevel::Ring3)
                            .readable_writable(true),
                        LimitFlags::new(),
                    ),
                    tss: SystemSegmentDescriptor64::default(),
                }
            }
            /// Load the TSS segment into the GDT
            pub fn load_tss(&mut self, tss: SystemSegmentDescriptor64) {
                self.tss = tss;
                let tss_selector = SegmentSelector::new()
                    .rpl(ProtectionLevel::Ring0)
                    .section(Sections::TaskStateSegment);
                unsafe {
                    instructions::ltr(tss_selector);
                }
            }
            /// Load the GDT with the `lgdt` instruction
            ///
            /// # Safety
            /// This function doesn't check if a GDT already exists, and just
            /// overrides it.
            pub unsafe fn load(&'static self) {
                let gdtr = {
                    GlobalDescriptorTableRegister {
                        limit: (size_of::<Self>() - 1) as u16,
                        base: self as *const _ as usize,
                    }
                };
                unsafe {
                    instructions::lgdt(&gdtr);
                }
            }
        }
        unsafe impl Send for GlobalDescriptorTableRegister {}
        unsafe impl Sync for GlobalDescriptorTableRegister {}
    }
    pub mod interrupt_descriptor_table {
        use common::{
            address_types::{Address, VirtualAddress},
            enums::{
                ProtectionLevel, Sections, SystemSegmentType,
                interrupts::{Interrupt, InterruptStackTable, InterruptType},
            },
        };
        use core::{arch::asm, panic};
        use core::{mem::MaybeUninit, ptr};
        use macros::bitfields;
        /// Global reference into the interrupt table
        pub static mut IDT: MaybeUninit<&mut InterruptDescriptorTable> = MaybeUninit::uninit();
        /// Global TSS segment
        pub static TSS: TaskStateSegment = TaskStateSegment::default();
        use crate::{
            instructions, registers::rflags::Rflags,
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
        #[repr(transparent)]
        pub struct InterruptAttributes(u16);
        #[automatically_derived]
        impl ::core::marker::Copy for InterruptAttributes {}
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for InterruptAttributes {}
        #[automatically_derived]
        impl ::core::clone::Clone for InterruptAttributes {
            #[inline]
            fn clone(&self) -> InterruptAttributes {
                let _: ::core::clone::AssertParamIsClone<u16>;
                *self
            }
        }
        impl InterruptAttributes {
            #[inline]
            pub const fn new() -> Self {
                Self(0)
            }
            #[inline]
            #[track_caller]
            fn get_ist(&self) -> u8 {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 3usize as u32)) << 0usize;
                    let bits = (val & mask) >> 0usize;
                    <u8 as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into u8")
                }
            }
            #[inline]
            #[track_caller]
            fn set_ist(&mut self, v: u8) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (u8) into u8");
                if true {
                    if !((v as u16) <= (7u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::set_ist: value out of range: must fit in 3 bits (max 0x7)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 3usize as u32)) << 0usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u16) << 0usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn ist(mut self, v: u8) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (u8) into u8");
                if true {
                    if !((v as u16) <= (7u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::ist: value out of range: must fit in 3 bits (max 0x7)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u16) << 0usize;
                self
            }
            #[inline]
            #[track_caller]
            fn get_reserved(&self) -> u8 {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 5usize as u32)) << 3usize;
                    let bits = (val & mask) >> 3usize;
                    <u8 as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into u8")
                }
            }
            #[inline]
            #[track_caller]
            fn get_int_type(&self) -> InterruptType {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 3usize as u32)) << 8usize;
                    let bits = (val & mask) >> 8usize;
                    <InterruptType as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into InterruptType")
                }
            }
            #[inline]
            #[track_caller]
            fn set_int_type(&mut self, v: InterruptType) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (InterruptType) into u8");
                if true {
                    if !((v as u16) <= (7u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::set_int_type: value out of range: must fit in 3 bits (max 0x7)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 3usize as u32)) << 8usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u16) << 8usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn int_type(mut self, v: InterruptType) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (InterruptType) into u8");
                if true {
                    if !((v as u16) <= (7u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::int_type: value out of range: must fit in 3 bits (max 0x7)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u16) << 8usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_zero(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 11usize;
                    let bits = (val & mask) >> 11usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn clear_zero(&mut self) {
                let v = 0usize as u8;
                if true {
                    if !((v as u16) <= (1u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::clear_zero: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 11usize;
                    let cleared = val & !mask;
                    let new = cleared | ((0usize as u16) << 11usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            fn get_dpl(&self) -> ProtectionLevel {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 2usize as u32)) << 12usize;
                    let bits = (val & mask) >> 12usize;
                    <ProtectionLevel as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into ProtectionLevel")
                }
            }
            #[inline]
            #[track_caller]
            fn set_dpl(&mut self, v: ProtectionLevel) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u16) <= (3u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::set_dpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 2usize as u32)) << 12usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u16) << 12usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn dpl(mut self, v: ProtectionLevel) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u16) <= (3u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::dpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u16) << 12usize;
                self
            }
            #[inline]
            #[track_caller]
            fn is_present(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 14usize;
                    let bits = (val & mask) >> 14usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            fn set_present(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u16) <= (1u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::set_present: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 14usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u16) << 14usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            const fn present(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u16) <= (1u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "InterruptAttributes::present: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u16) << 14usize;
                self
            }
        }
        impl const ::core::convert::From<u16> for InterruptAttributes {
            fn from(value: u16) -> Self {
                InterruptAttributes(value)
            }
        }
        impl const ::core::convert::From<InterruptAttributes> for u16 {
            fn from(value: InterruptAttributes) -> Self {
                value.0
            }
        }
        impl ::core::fmt::Debug for InterruptAttributes {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("InterruptAttributes")
                    .field("ist", &self.get_ist())
                    .field("reserved", &self.get_reserved())
                    .field("int_type", &self.get_int_type())
                    .field("zero", &self.is_zero())
                    .field("dpl", &self.get_dpl())
                    .field("present", &self.is_present())
                    .finish()
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
                let mut gdt_register: MaybeUninit<GlobalDescriptorTableRegister> = MaybeUninit::uninit();
                let gdt = unsafe {
                    asm!(
                        "sgdt [{0}]", in (reg) gdt_register.as_mut_ptr(),
                        options(preserves_flags, nostack)
                    );
                    &mut *(gdt_register.assume_init().base
                        as *mut GlobalDescriptorTableLong)
                };
                if TSS.iomb() < size_of::<TaskStateSegment>() as u16 {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "I/O maps are not supported, change TSS IOMB into number larger then 0x68",
                            ),
                        );
                    }
                }
                let tss = SystemSegmentDescriptor64::new(
                    &TSS as *const _ as u64,
                    (size_of::<TaskStateSegment>() - 1) as u32,
                    SystemSegmentType::TaskStateSegmentAvailable,
                );
                gdt.load_tss(tss);
                unsafe {
                    ptr::write_volatile(
                        base_address.as_non_null::<InterruptDescriptorTable>().as_ptr(),
                        InterruptDescriptorTable {
                            interrupts: [const {
                                InterruptDescriptorTableEntry::missing()
                            }; 256],
                        },
                    );
                    uninit
                        .write(
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
                    InterruptAttributes::new()
                        .present(true)
                        .dpl(dpl)
                        .int_type(handler_type),
                    SegmentSelector::new()
                        .rpl(ProtectionLevel::Ring0)
                        .section(Sections::KernelCode),
                );
                self.interrupts[routine as usize] = entry;
            }
        }
        /// Entry structure in the Interrupt Descriptor Table
        #[repr(C, packed)]
        pub struct InterruptDescriptorTableEntry {
            handler_offset_low: u16,
            segment_selector: SegmentSelector,
            ist: InterruptStackTable,
            attributes: InterruptAttributes,
            handler_offset_mid: u16,
            handler_offset_high: u32,
            zero: u32,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for InterruptDescriptorTableEntry {
            #[inline]
            fn clone(&self) -> InterruptDescriptorTableEntry {
                InterruptDescriptorTableEntry {
                    handler_offset_low: ::core::clone::Clone::clone(
                        &{ self.handler_offset_low },
                    ),
                    segment_selector: ::core::clone::Clone::clone(
                        &{ self.segment_selector },
                    ),
                    ist: ::core::clone::Clone::clone(&{ self.ist }),
                    attributes: ::core::clone::Clone::clone(&{ self.attributes }),
                    handler_offset_mid: ::core::clone::Clone::clone(
                        &{ self.handler_offset_mid },
                    ),
                    handler_offset_high: ::core::clone::Clone::clone(
                        &{ self.handler_offset_high },
                    ),
                    zero: ::core::clone::Clone::clone(&{ self.zero }),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for InterruptDescriptorTableEntry {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "handler_offset_low",
                    "segment_selector",
                    "ist",
                    "attributes",
                    "handler_offset_mid",
                    "handler_offset_high",
                    "zero",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &{ self.handler_offset_low },
                    &{ self.segment_selector },
                    &{ self.ist },
                    &{ self.attributes },
                    &{ self.handler_offset_mid },
                    &{ self.handler_offset_high },
                    &&{ self.zero },
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "InterruptDescriptorTableEntry",
                    names,
                    values,
                )
            }
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
                let handler_offset_high = (handler_address.as_usize() >> 32) as u32;
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
        pub struct InterruptStackFrame {
            pub instruction_pointer: VirtualAddress,
            pub code_segment: usize,
            pub cpu_flags: Rflags,
            pub stack_pointer: VirtualAddress,
            pub stack_segment: usize,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for InterruptStackFrame {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field5_finish(
                    f,
                    "InterruptStackFrame",
                    "instruction_pointer",
                    &self.instruction_pointer,
                    "code_segment",
                    &self.code_segment,
                    "cpu_flags",
                    &self.cpu_flags,
                    "stack_pointer",
                    &self.stack_pointer,
                    "stack_segment",
                    &&self.stack_segment,
                )
            }
        }
    }
    pub mod mbr {
        #[repr(C, packed)]
        pub struct PartitionTableEntry {
            /// Boot indicator bit flag: 0 = no, 0x80 = bootable
            /// (or "active").
            pub bootable: u8,
            /// Starting head of the partition.
            pub start_head: u8,
            /// Bits 0-5 are the starting sector.
            /// Bits 6-16 are the starting cylinder.
            pub sector_cylinder_start: u16,
            /// SystemID.
            pub system_id: u8,
            /// Ending head of the partition.
            pub end_head: u8,
            /// Bits 0-5 are the ending sector.
            /// Bits 6-16 are the ending cylinder.
            pub sector_cylinder_head: u16,
            /// Relative Sector (to start of partition -- also
            /// equals the partition's starting LBA value)
            pub relative_sector: u32,
            /// Total Sectors in partition
            pub total_sectors: u32,
        }
        pub struct MasterBootRecord {
            pub entries: [PartitionTableEntry; 4],
        }
    }
    pub mod paging {
        #[macro_use]
        pub mod entry_flags {
            use macros::bitfields;
            /// A wrapper for `PageTableEntry` flags for easier use
            #[repr(transparent)]
            pub struct PageEntryFlags(u16);
            #[automatically_derived]
            impl ::core::marker::Copy for PageEntryFlags {}
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for PageEntryFlags {}
            #[automatically_derived]
            impl ::core::clone::Clone for PageEntryFlags {
                #[inline]
                fn clone(&self) -> PageEntryFlags {
                    let _: ::core::clone::AssertParamIsClone<u16>;
                    *self
                }
            }
            impl PageEntryFlags {
                #[inline]
                pub const fn new() -> Self {
                    Self(0)
                }
                #[inline]
                #[track_caller]
                pub fn is_present(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 0usize;
                        let bits = (val & mask) >> 0usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_present(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_present: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 0usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 0usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn present(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::present: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 0usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_writable(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 1usize;
                        let bits = (val & mask) >> 1usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_writable(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_writable: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 1usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 1usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn writable(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::writable: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 1usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_usr_access(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 2usize;
                        let bits = (val & mask) >> 2usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_usr_access(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_usr_access: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 2usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 2usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn usr_access(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::usr_access: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 2usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_write_through_cache(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 3usize;
                        let bits = (val & mask) >> 3usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_write_through_cache(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_write_through_cache: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 3usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 3usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn write_through_cache(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::write_through_cache: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 3usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_disable_cache(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 4usize;
                        let bits = (val & mask) >> 4usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_disable_cache(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_disable_cache: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 4usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 4usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn disable_cache(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::disable_cache: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 4usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_accessed(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 5usize;
                        let bits = (val & mask) >> 5usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn is_dirty(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 6usize;
                        let bits = (val & mask) >> 6usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn is_huge_page(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 7usize;
                        let bits = (val & mask) >> 7usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_huge_page(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_huge_page: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 7usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 7usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn huge_page(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::huge_page: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 7usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_global(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 8usize;
                        let bits = (val & mask) >> 8usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_global(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_global: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 8usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 8usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn global(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::global: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 8usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_full(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 9usize;
                        let bits = (val & mask) >> 9usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_full(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_full: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 9usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 9usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn full(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::full: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 9usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_table(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 10usize;
                        let bits = (val & mask) >> 10usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_table(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_table: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 10usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 10usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn table(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::table: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 10usize;
                    self
                }
                #[inline]
                #[track_caller]
                pub fn is_root_entry(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 11usize;
                        let bits = (val & mask) >> 11usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                pub fn set_root_entry(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::set_root_entry: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u16;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 11usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u16) << 11usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                pub const fn root_entry(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u16) <= (1u128 as u16)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageEntryFlags::root_entry: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u16) << 11usize;
                    self
                }
            }
            impl const ::core::convert::From<u16> for PageEntryFlags {
                fn from(value: u16) -> Self {
                    PageEntryFlags(value)
                }
            }
            impl const ::core::convert::From<PageEntryFlags> for u16 {
                fn from(value: PageEntryFlags) -> Self {
                    value.0
                }
            }
            impl ::core::fmt::Debug for PageEntryFlags {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("PageEntryFlags")
                        .field("present", &self.is_present())
                        .field("writable", &self.is_writable())
                        .field("usr_access", &self.is_usr_access())
                        .field("write_through_cache", &self.is_write_through_cache())
                        .field("disable_cache", &self.is_disable_cache())
                        .field("accessed", &self.is_accessed())
                        .field("dirty", &self.is_dirty())
                        .field("huge_page", &self.is_huge_page())
                        .field("global", &self.is_global())
                        .field("full", &self.is_full())
                        .field("table", &self.is_table())
                        .field("root_entry", &self.is_root_entry())
                        .finish()
                }
            }
            impl PageEntryFlags {
                /// Default flags for entry that contains page table.
                pub const fn table_flags() -> Self {
                    PageEntryFlags::new().present(true).writable(true).table(true)
                }
                /// Default flags for entry that contains huge page.
                pub const fn huge_page_flags() -> Self {
                    PageEntryFlags::new().present(true).writable(true).huge_page(true)
                }
                /// Default flags for entry that contains regular page.
                pub const fn regular_page_flags() -> Self {
                    PageEntryFlags::new().present(true).writable(true)
                }
                pub const fn regular_io_page_flags() -> Self {
                    PageEntryFlags::new()
                        .present(true)
                        .writable(true)
                        .disable_cache(true)
                        .global(true)
                }
                pub const fn huge_io_page_flags() -> Self {
                    PageEntryFlags::new()
                        .present(true)
                        .writable(true)
                        .huge_page(true)
                        .disable_cache(true)
                        .global(true)
                }
            }
        }
        pub mod init {
            use super::{PageEntryFlags, PageTable};
            use common::address_types::Address;
            use common::{
                address_types::PhysicalAddress,
                constants::{
                    IDENTITY_PAGE_TABLE_L2_OFFSET, IDENTITY_PAGE_TABLE_L3_OFFSET,
                    IDENTITY_PAGE_TABLE_L4_OFFSET, TOP_IDENTITY_PAGE_TABLE_L2_OFFSET,
                    TOP_IDENTITY_PAGE_TABLE_L3_OFFSET,
                },
            };
            use core::arch::asm;
            fn init_identity_tables() -> Option<&'static mut PageTable> {
                let identity_page_table_l4 = unsafe {
                    PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L4_OFFSET.into())?
                        .as_mut()
                };
                let identity_page_table_l3 = unsafe {
                    PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L3_OFFSET.into())?
                        .as_mut()
                };
                let identity_page_table_l2 = unsafe {
                    PageTable::empty_from_ptr(IDENTITY_PAGE_TABLE_L2_OFFSET.into())?
                        .as_mut()
                };
                unsafe {
                    identity_page_table_l4
                        .entries[0]
                        .map_unchecked(
                            PhysicalAddress::new_unchecked(
                                IDENTITY_PAGE_TABLE_L3_OFFSET,
                            ),
                            PageEntryFlags::table_flags(),
                        );
                    identity_page_table_l3
                        .entries[0]
                        .map_unchecked(
                            PhysicalAddress::new_unchecked(
                                IDENTITY_PAGE_TABLE_L2_OFFSET,
                            ),
                            PageEntryFlags::table_flags(),
                        );
                    identity_page_table_l2
                        .entries[0]
                        .map_unchecked(
                            PhysicalAddress::new_unchecked(0),
                            PageEntryFlags::huge_page_flags(),
                        );
                }
                Some(identity_page_table_l4)
            }
            fn init_kernel_tables(pml4: &mut PageTable) -> Option<()> {
                let top_identity_page_table_l3 = unsafe {
                    PageTable::empty_from_ptr(TOP_IDENTITY_PAGE_TABLE_L3_OFFSET.into())?
                        .as_mut()
                };
                let top_identity_page_table_l2 = unsafe {
                    PageTable::empty_from_ptr(TOP_IDENTITY_PAGE_TABLE_L2_OFFSET.into())?
                        .as_mut()
                };
                unsafe {
                    pml4.entries[256]
                        .map_unchecked(
                            PhysicalAddress::new_unchecked(
                                TOP_IDENTITY_PAGE_TABLE_L3_OFFSET,
                            ),
                            PageEntryFlags::table_flags(),
                        );
                    top_identity_page_table_l3
                        .entries[0]
                        .map_unchecked(
                            PhysicalAddress::new_unchecked(
                                TOP_IDENTITY_PAGE_TABLE_L2_OFFSET,
                            ),
                            PageEntryFlags::table_flags(),
                        );
                    top_identity_page_table_l2
                        .entries[0]
                        .map_unchecked(
                            PhysicalAddress::new_unchecked(0),
                            PageEntryFlags::huge_io_page_flags(),
                        );
                }
                Some(())
            }
            fn set_cr3() {
                unsafe {
                    asm!(
                        "mov eax, {0}\nmov cr3, eax", const IDENTITY_PAGE_TABLE_L4_OFFSET
                    );
                }
            }
            fn set_pae_long_mode() {
                unsafe {
                    asm!("mov eax, cr4\nor eax, 1 << 5\nmov cr4, eax");
                    asm!("mov ecx, 0xC0000080\nrdmsr\nor eax, 1 << 8\nwrmsr");
                }
            }
            fn toggle_paging() {
                unsafe {
                    asm!("mov eax, cr0\nor eax, 1 << 31\nmov cr0, eax");
                }
            }
        }
        pub mod page_table {
            use core::ptr::{self, NonNull};
            use crate::{registers::cr3, structures::paging::PageTableEntry};
            use common::{
                address_types::{Address, VirtualAddress},
                constants::{PAGE_DIRECTORY_ENTRIES, REGULAR_PAGE_ALIGNMENT},
                enums::{PageSize, PageTableLevel},
                error::EntryError,
            };
            #[repr(C)]
            #[repr(align(4096))]
            pub struct PageTable {
                pub entries: [PageTableEntry; PAGE_DIRECTORY_ENTRIES],
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for PageTable {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "PageTable",
                        "entries",
                        &&self.entries,
                    )
                }
            }
            pub enum EntryIndex {
                Entry(&'static PageTableEntry),
                Index(usize),
                PageDoesNotFit,
                OutOfEntries,
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for EntryIndex {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match self {
                        EntryIndex::Entry(__self_0) => {
                            ::core::fmt::Formatter::debug_tuple_field1_finish(
                                f,
                                "Entry",
                                &__self_0,
                            )
                        }
                        EntryIndex::Index(__self_0) => {
                            ::core::fmt::Formatter::debug_tuple_field1_finish(
                                f,
                                "Index",
                                &__self_0,
                            )
                        }
                        EntryIndex::PageDoesNotFit => {
                            ::core::fmt::Formatter::write_str(f, "PageDoesNotFit")
                        }
                        EntryIndex::OutOfEntries => {
                            ::core::fmt::Formatter::write_str(f, "OutOfEntries")
                        }
                    }
                }
            }
            impl PageTable {
                /// Create an empty page table
                #[inline]
                pub const fn empty() -> Self {
                    Self {
                        entries: { [PageTableEntry::new(); PAGE_DIRECTORY_ENTRIES] },
                    }
                }
                /// Create an empty page table at the given virtual address
                ///
                /// # Safety
                /// This function works on every address, and will override the data at
                /// that address
                #[inline]
                pub unsafe fn empty_from_ptr(
                    page_table_ptr: VirtualAddress,
                ) -> Option<NonNull<PageTable>> {
                    if !page_table_ptr.is_aligned(REGULAR_PAGE_ALIGNMENT) {
                        return None;
                    }
                    unsafe {
                        ptr::write_volatile(
                            page_table_ptr.as_non_null::<PageTable>().as_ptr(),
                            PageTable::empty(),
                        );
                        Some(page_table_ptr.as_non_null::<PageTable>())
                    }
                }
                #[inline]
                pub fn current_table() -> NonNull<PageTable> {
                    NonNull::new(cr3::read() as usize as *mut PageTable)
                        .expect("Page table pointer is not present in cr3, found NULL")
                }
                #[inline]
                pub fn address(&self) -> VirtualAddress {
                    unsafe {
                        VirtualAddress::new_unchecked(self as *const Self as usize)
                    }
                }
                /// Tries to fetch a page table entry or an empty page starting from
                /// the given index.
                ///
                /// Returns the index of the found entry and the page table if found.
                pub fn try_fetch_table(
                    &'static self,
                    start_at: usize,
                    table_level: PageTableLevel,
                    page_size: PageSize,
                ) -> EntryIndex {
                    if !page_size.allocatable_at(table_level) {
                        return EntryIndex::PageDoesNotFit;
                    }
                    for (i, entry) in self.entries.iter().enumerate().skip(start_at) {
                        match entry.mapped_table() {
                            Ok(_) => {
                                return EntryIndex::Entry(entry);
                            }
                            Err(EntryError::NoMapping) => {
                                return EntryIndex::Index(i);
                            }
                            Err(EntryError::NotATable) => continue,
                        }
                    }
                    EntryIndex::OutOfEntries
                }
            }
        }
        pub mod page_table_entry {
            use core::ptr::NonNull;
            use crate::structures::paging::{PageEntryFlags, PageTable};
            use common::{
                address_types::{Address, PhysicalAddress},
                constants::REGULAR_PAGE_ALIGNMENT, error::EntryError,
            };
            use macros::bitfields;
            #[repr(transparent)]
            pub struct PageTableEntry(u64);
            #[automatically_derived]
            impl ::core::marker::Copy for PageTableEntry {}
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for PageTableEntry {}
            #[automatically_derived]
            impl ::core::clone::Clone for PageTableEntry {
                #[inline]
                fn clone(&self) -> PageTableEntry {
                    let _: ::core::clone::AssertParamIsClone<u64>;
                    *self
                }
            }
            impl PageTableEntry {
                #[inline]
                pub const fn new() -> Self {
                    Self(0)
                }
                #[inline]
                #[track_caller]
                fn get_flags(&self) -> PageEntryFlags {
                    unsafe {
                        let addr = self as *const _ as *mut u64;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u64::MAX >> (u64::BITS - 12usize as u32)) << 0usize;
                        let bits = (val & mask) >> 0usize;
                        <PageEntryFlags as ::core::convert::TryFrom<
                            u16,
                        >>::try_from(bits as u16)
                            .expect(
                                "Cannot convert bit representation into PageEntryFlags",
                            )
                    }
                }
                #[inline]
                #[track_caller]
                fn set_flags(&mut self, v: PageEntryFlags) {
                    let v = <u16 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (PageEntryFlags) into u16");
                    if true {
                        if !((v as u64) <= (4095u128 as u64)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::set_flags: value out of range: must fit in 12 bits (max 0xfff)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u64;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u64::MAX >> (u64::BITS - 12usize as u32)) << 0usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u64) << 0usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                fn clear_flags(&mut self) {
                    let v = 0usize as u16;
                    if true {
                        if !((v as u64) <= (4095u128 as u64)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::clear_flags: value out of range: must fit in 12 bits (max 0xfff)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u64;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u64::MAX >> (u64::BITS - 12usize as u32)) << 0usize;
                        let cleared = val & !mask;
                        let new = cleared | ((0usize as u64) << 0usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                const fn flags(mut self, v: PageEntryFlags) -> Self {
                    let v = <u16 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (PageEntryFlags) into u16");
                    if true {
                        if !((v as u64) <= (4095u128 as u64)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::flags: value out of range: must fit in 12 bits (max 0xfff)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u64) << 0usize;
                    self
                }
                #[inline]
                #[track_caller]
                fn get_address(&self) -> PhysicalAddress {
                    unsafe {
                        let addr = self as *const _ as *mut u64;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u64::MAX >> (u64::BITS - 51usize as u32)) << 12usize;
                        let bits = (val & mask);
                        <PhysicalAddress as ::core::convert::TryFrom<
                            u64,
                        >>::try_from(bits as u64)
                            .expect(
                                "Cannot convert bit representation into PhysicalAddress",
                            )
                    }
                }
                #[inline]
                #[track_caller]
                fn set_address(&mut self, v: PhysicalAddress) {
                    let v = <u64 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (PhysicalAddress) into u64");
                    if true {
                        if !((v as u64) <= (2251799813685247u128 as u64)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::set_address: value out of range: must fit in 51 bits (max 0x7ffffffffffff)",
                                    ),
                                );
                            }
                        }
                    }
                    if true {
                        if !(v & !(((2251799813685247u128) as u64) << 12usize) == 0) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::set_address: value contains bits outside the 51-bit field at bit offset 12 (permitted mask: 0x7ffffffffffff000)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u64;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u64::MAX >> (u64::BITS - 51usize as u32)) << 12usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u64));
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                const fn address(mut self, v: PhysicalAddress) -> Self {
                    let v = <u64 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (PhysicalAddress) into u64");
                    if true {
                        if !((v as u64) <= (2251799813685247u128 as u64)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::address: value out of range: must fit in 51 bits (max 0x7ffffffffffff)",
                                    ),
                                );
                            }
                        }
                    }
                    if true {
                        if !(v & !(((2251799813685247u128) as u64) << 12usize) == 0) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::address: value contains bits outside the 51-bit field at bit offset 12 (permitted mask: 0x7ffffffffffff000)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u64);
                    self
                }
                #[inline]
                #[track_caller]
                fn is_not_executable(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut u64;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 63usize;
                        let bits = (val & mask) >> 63usize;
                        <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                            .expect("Cannot convert bit representation into bool")
                    }
                }
                #[inline]
                #[track_caller]
                fn set_not_executable(&mut self, v: bool) {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u64) <= (1u128 as u64)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::set_not_executable: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    unsafe {
                        let addr = self as *const _ as *mut u64;
                        let val = ::core::ptr::read_volatile(addr);
                        let mask = (u64::MAX >> (u64::BITS - 1usize as u32)) << 63usize;
                        let cleared = val & !mask;
                        let new = cleared | ((v as u64) << 63usize);
                        ::core::ptr::write_volatile(addr, new);
                    }
                }
                #[inline]
                #[track_caller]
                const fn not_executable(mut self, v: bool) -> Self {
                    let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                        .ok()
                        .expect("Can't convert value 'v' (bool) into u8");
                    if true {
                        if !((v as u64) <= (1u128 as u64)) {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!(
                                        "PageTableEntry::not_executable: value out of range: must fit in 1 bits (max 0x1)",
                                    ),
                                );
                            }
                        }
                    }
                    self.0 |= (v as u64) << 63usize;
                    self
                }
            }
            impl const ::core::convert::From<u64> for PageTableEntry {
                fn from(value: u64) -> Self {
                    PageTableEntry(value)
                }
            }
            impl const ::core::convert::From<PageTableEntry> for u64 {
                fn from(value: PageTableEntry) -> Self {
                    value.0
                }
            }
            impl ::core::fmt::Debug for PageTableEntry {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("PageTableEntry")
                        .field("flags", &self.get_flags())
                        .field("address", &self.get_address())
                        .field("not_executable", &self.is_not_executable())
                        .finish()
                }
            }
            impl PageTableEntry {
                /// Map new frame with the given with the given flags.
                ///
                /// # Safety
                ///
                /// This function doesn't check if address is properly aligned, and if
                /// the entry was already mapped.
                #[inline]
                pub unsafe fn map_unchecked(
                    &mut self,
                    frame: PhysicalAddress,
                    flags: PageEntryFlags,
                ) {
                    self.set_flags(flags.present(true));
                    self.set_address(frame);
                }
                /// Map a frame to the page table entry while checking
                /// flags and frame alignment but **not** the ownership
                /// of the frame address This function **will** set
                /// the entry as present even if it was not specified in
                /// the flags.
                ///
                /// # Parameters
                ///
                /// - `frame`: The physical address of the mapped frame
                ///
                /// # Interrupts
                /// This function will raise a PAGE_FAULT if the entry
                /// is already mapped
                ///
                /// # Safety
                /// The `frame` address should not be used by anyone
                /// except the corresponding virtual address,
                /// and should be marked owned by it in a memory
                /// allocator
                #[inline]
                pub unsafe fn map(
                    &mut self,
                    frame: PhysicalAddress,
                    flags: PageEntryFlags,
                ) {
                    if !self.get_flags().is_present()
                        && frame.is_aligned(REGULAR_PAGE_ALIGNMENT)
                    {
                        unsafe { self.map_unchecked(frame, flags) };
                    }
                }
                /// Return the physical address that is mapped by this
                /// entry, if this entry is not mapped, return None.
                #[inline]
                pub fn mapped(&self) -> Result<PhysicalAddress, EntryError> {
                    if self.get_flags().is_present() {
                        Ok(self.get_address())
                    } else {
                        Err(EntryError::NoMapping)
                    }
                }
                /// Return the physical address mapped by this table as
                /// a reference into a page table.
                ///
                /// This method assumes all page tables are identity
                /// mapped.
                pub fn mapped_table(&self) -> Result<NonNull<PageTable>, EntryError> {
                    let pt = self.mapped()?.translate().as_non_null::<PageTable>();
                    let flags = self.get_flags();
                    if !flags.is_huge_page() && flags.is_table() {
                        Ok(pt)
                    } else {
                        Err(EntryError::NotATable)
                    }
                }
                pub fn table_index(&self) -> usize {
                    let table_offset = self as *const _ as usize & ((1 << 12) - 1);
                    table_offset / size_of::<PageTableEntry>()
                }
            }
        }
        pub use entry_flags::*;
        pub use page_table::*;
        pub use page_table_entry::*;
    }
    pub mod segments {
        use common::{
            address_types::{Address, VirtualAddress},
            enums::{ProtectionLevel, Sections},
        };
        use macros::bitfields;
        #[repr(transparent)]
        pub struct SegmentSelector(u16);
        #[automatically_derived]
        impl ::core::marker::Copy for SegmentSelector {}
        #[automatically_derived]
        #[doc(hidden)]
        unsafe impl ::core::clone::TrivialClone for SegmentSelector {}
        #[automatically_derived]
        impl ::core::clone::Clone for SegmentSelector {
            #[inline]
            fn clone(&self) -> SegmentSelector {
                let _: ::core::clone::AssertParamIsClone<u16>;
                *self
            }
        }
        impl SegmentSelector {
            #[inline]
            pub const fn new() -> Self {
                Self(0)
            }
            #[inline]
            #[track_caller]
            pub fn get_rpl(&self) -> ProtectionLevel {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 2usize as u32)) << 0usize;
                    let bits = (val & mask) >> 0usize;
                    <ProtectionLevel as ::core::convert::TryFrom<
                        u8,
                    >>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into ProtectionLevel")
                }
            }
            #[inline]
            #[track_caller]
            pub fn set_rpl(&mut self, v: ProtectionLevel) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u16) <= (3u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SegmentSelector::set_rpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 2usize as u32)) << 0usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u16) << 0usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            pub const fn rpl(mut self, v: ProtectionLevel) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (ProtectionLevel) into u8");
                if true {
                    if !((v as u16) <= (3u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SegmentSelector::rpl: value out of range: must fit in 2 bits (max 0x3)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u16) << 0usize;
                self
            }
            #[inline]
            #[track_caller]
            pub fn is_use_ldt(&self) -> bool {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 2usize;
                    let bits = (val & mask) >> 2usize;
                    <bool as ::core::convert::TryFrom<u8>>::try_from(bits as u8)
                        .expect("Cannot convert bit representation into bool")
                }
            }
            #[inline]
            #[track_caller]
            pub fn set_use_ldt(&mut self, v: bool) {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u16) <= (1u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SegmentSelector::set_use_ldt: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 1usize as u32)) << 2usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u16) << 2usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            pub const fn use_ldt(mut self, v: bool) -> Self {
                let v = <u8 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (bool) into u8");
                if true {
                    if !((v as u16) <= (1u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SegmentSelector::use_ldt: value out of range: must fit in 1 bits (max 0x1)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u16) << 2usize;
                self
            }
            #[inline]
            #[track_caller]
            pub fn get_section(&self) -> Sections {
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 13usize as u32)) << 3usize;
                    let bits = (val & mask) >> 3usize;
                    <Sections as ::core::convert::TryFrom<u16>>::try_from(bits as u16)
                        .expect("Cannot convert bit representation into Sections")
                }
            }
            #[inline]
            #[track_caller]
            pub fn set_section(&mut self, v: Sections) {
                let v = <u16 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (Sections) into u16");
                if true {
                    if !((v as u16) <= (8191u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SegmentSelector::set_section: value out of range: must fit in 13 bits (max 0x1fff)",
                                ),
                            );
                        }
                    }
                }
                unsafe {
                    let addr = self as *const _ as *mut u16;
                    let val = ::core::ptr::read_volatile(addr);
                    let mask = (u16::MAX >> (u16::BITS - 13usize as u32)) << 3usize;
                    let cleared = val & !mask;
                    let new = cleared | ((v as u16) << 3usize);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
            #[inline]
            #[track_caller]
            pub const fn section(mut self, v: Sections) -> Self {
                let v = <u16 as ::core::convert::TryFrom<_>>::try_from(v)
                    .ok()
                    .expect("Can't convert value 'v' (Sections) into u16");
                if true {
                    if !((v as u16) <= (8191u128 as u16)) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "SegmentSelector::section: value out of range: must fit in 13 bits (max 0x1fff)",
                                ),
                            );
                        }
                    }
                }
                self.0 |= (v as u16) << 3usize;
                self
            }
        }
        impl const ::core::convert::From<u16> for SegmentSelector {
            fn from(value: u16) -> Self {
                SegmentSelector(value)
            }
        }
        impl const ::core::convert::From<SegmentSelector> for u16 {
            fn from(value: SegmentSelector) -> Self {
                value.0
            }
        }
        impl ::core::fmt::Debug for SegmentSelector {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("SegmentSelector")
                    .field("rpl", &self.get_rpl())
                    .field("use_ldt", &self.is_use_ldt())
                    .field("section", &self.get_section())
                    .finish()
            }
        }
        /// Structure of the Task State Segment
        #[repr(C, packed)]
        pub struct TaskStateSegment {
            _reserved0: u32,
            /// Privileged stack pointers that can be used on
            /// interrupt from higher privilege
            priv_stack_ptr: [VirtualAddress; 3],
            _reserved1: u64,
            int_stack_table: [VirtualAddress; 7],
            _reserved2: u64,
            _reserved3: u16,
            /// An offset from the base address of this struct to
            /// the I/O map
            io_map_offset: u16,
        }
        impl TaskStateSegment {
            /// Return the I/O map base address
            pub const fn iomb(&self) -> u16 {
                self.io_map_offset
            }
            /// Construct a default TSS
            pub const fn default() -> Self {
                Self {
                    _reserved0: 0,
                    _reserved1: 0,
                    _reserved2: 0,
                    _reserved3: 0,
                    priv_stack_ptr: [unsafe { VirtualAddress::new_unchecked(0) }; 3],
                    int_stack_table: [unsafe { VirtualAddress::new_unchecked(0) }; 7],
                    io_map_offset: size_of::<Self>() as u16,
                }
            }
        }
    }
}
