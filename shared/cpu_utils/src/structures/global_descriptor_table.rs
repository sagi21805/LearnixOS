use common::{
    enums::{ProtectionLevel, SystemSegmentType},
    flag,
};
use core::arch::asm;

use crate::structures::segments::SegmentSelector;

struct AccessByte(u8);

impl AccessByte {
    /// Creates an access byte with all flags turned off.
    pub const fn new() -> Self {
        Self(0)
    }

    // Is this a valid segment?
    // for all active segments this should be turned on.
    flag!(present, 7);

    /// Sets the privilege level while returning self.
    /// This is corresponding to the cpu ring of this segment
    /// 0 is commonly called kernel mode, 4 is commonly called user mode
    pub const fn dpl(mut self, level: ProtectionLevel) -> Self {
        self.0 |= (level as u8) << 5;
        self
    }

    /// Set the type for a system segment.
    ///
    /// **Note:** This function is relevant only for system segments
    pub const fn set_system_type(mut self, system_type: SystemSegmentType) -> Self {
        self.0 |= system_type as u8;
        self
    }

    // Is this a code / data segment or a system segment.
    flag!(code_or_data, 4);
    // Will this segment contains executable code?
    flag!(executable, 3);
    // Will the segment grow downwards?
    // relevant for non executable segments
    flag!(direction, 2);
    // Can this code be executed from lower privilege segments.
    // relevant to executable segments
    flag!(conforming, 2);
    // Can this segment be read or it is only executable?
    // relevant for code segment
    flag!(readable, 1);
    // Is this segment writable?
    // relevant for data segments
    flag!(writable, 1);
}

struct LimitFlags(u8);

impl LimitFlags {
    /// Creates a default limit flags with all flags turned off.
    pub const fn new() -> Self {
        Self(0)
    }
    // Toggle on paging for this segment (limit *= 0x1000)
    flag!(granularity, 7);
    // Is this segment going to use 32bit mode?
    flag!(protected, 6);
    // Set long mode flag, this will also clear protected mode
    flag!(long, 5);
}

#[repr(C)]
struct GlobalDescriptorTableEntry32 {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access_byte: AccessByte,
    /// Low 4 bits limit high 4 bits flags
    limit_flags: LimitFlags,
    base_high: u8,
}

impl GlobalDescriptorTableEntry32 {
    pub const fn empty() -> Self {
        Self {
            limit_flags: LimitFlags::new(),
            access_byte: AccessByte::new(),
            base_high: 0,
            base_low: 0,
            base_mid: 0,
            limit_low: 0,
        }
    }

    pub const fn new(base: u32, limit: u32, access_byte: AccessByte, flags: LimitFlags) -> Self {
        let base_low = (base & 0xffff) as u16;
        let base_mid = ((base >> 0x10) & 0xff) as u8;
        let base_high = ((base >> 0x18) & 0xff) as u8;
        let limit_low = (limit & 0xffff) as u16;
        let limit_high = ((limit >> 0x10) & 0xf) as u8;
        let limit_flags = flags.0 | limit_high;
        Self {
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

#[repr(C, packed)]
pub struct SystemSegmentDescriptor64 {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access_byte: AccessByte,
    limit_flags: LimitFlags,
    base_high: u8,
    base_extra: u32,
    _reserved: u32,
}

impl SystemSegmentDescriptor64 {
    pub const fn empty() -> Self {
        SystemSegmentDescriptor64 {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access_byte: AccessByte::new(),
            limit_flags: LimitFlags::new(),
            base_high: 0,
            base_extra: 0,
            _reserved: 0,
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub const fn new(base: u64, limit: u32, segment_type: SystemSegmentType) -> Self {
        let base_low = (base & 0xffff) as u16;
        let base_mid = ((base >> 16) & 0xff) as u8;
        let base_high = ((base >> 24) & 0xff) as u8;
        let limit_low = (limit & 0xffff) as u16;
        let limit_high = ((limit >> 16) & 0xf) as u8;
        let base_extra = (base >> 32) as u32;

        let access_byte = AccessByte::new()
            .present()
            .dpl(ProtectionLevel::Ring0)
            .set_system_type(segment_type);

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

#[repr(C)]
pub struct GlobalDescriptorTableProtected {
    null: GlobalDescriptorTableEntry32,
    code: GlobalDescriptorTableEntry32,
    data: GlobalDescriptorTableEntry32,
}

impl GlobalDescriptorTableProtected {
    /// Creates default global descriptor table for protected mode
    pub const fn default() -> Self {
        Self {
            null: GlobalDescriptorTableEntry32::empty(),
            code: GlobalDescriptorTableEntry32::new(
                0,
                0xfffff,
                AccessByte::new()
                    .present()
                    .dpl(ProtectionLevel::Ring0)
                    .code_or_data()
                    .executable()
                    .readable(),
                LimitFlags::new().granularity().protected(),
            ),
            data: GlobalDescriptorTableEntry32::new(
                0,
                0xfffff,
                AccessByte::new()
                    .present()
                    .dpl(ProtectionLevel::Ring0)
                    .code_or_data()
                    .writable(),
                LimitFlags::new().granularity().protected(),
            ),
        }
    }

    pub unsafe fn load(&'static self) {
        let global_descriptor_table_register = {
            GlobalDescriptorTableRegister {
                limit: (size_of::<Self>() - 1) as u16,
                base: self as *const _ as usize,
            }
        };
        unsafe {
            asm!(
                "cli",
                "lgdt [{}]",
                in(reg) &global_descriptor_table_register,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}

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
    /// Creates default global descriptor table for long mode
    pub const fn default() -> Self {
        Self {
            null: GlobalDescriptorTableEntry32::empty(),
            kernel_code: GlobalDescriptorTableEntry32::new(
                0,
                0,
                AccessByte::new()
                    .code_or_data()
                    .present()
                    .dpl(ProtectionLevel::Ring0)
                    .writable()
                    .executable(),
                LimitFlags::new().long(),
            ),
            kernel_data: GlobalDescriptorTableEntry32::new(
                0,
                0,
                AccessByte::new()
                    .code_or_data()
                    .present()
                    .dpl(ProtectionLevel::Ring0)
                    .writable(),
                LimitFlags::new(),
            ),
            user_code: GlobalDescriptorTableEntry32::new(
                0,
                0,
                AccessByte::new()
                    .code_or_data()
                    .present()
                    .dpl(ProtectionLevel::Ring3)
                    .writable()
                    .executable(),
                LimitFlags::new().long(),
            ),
            user_data: GlobalDescriptorTableEntry32::new(
                0,
                0,
                AccessByte::new()
                    .code_or_data()
                    .present()
                    .dpl(ProtectionLevel::Ring3)
                    .writable(),
                LimitFlags::new(),
            ),
            tss: SystemSegmentDescriptor64::empty(),
        }
    }

    pub fn load_tss(&mut self, tss: SystemSegmentDescriptor64) {
        self.tss = tss;
        let tss_selector = SegmentSelector::new().set_table_index(5);
        unsafe {
            asm!(
                "ltr {0:x}",
                in(reg) tss_selector.as_u16()
            )
        }
    }

    pub unsafe fn load(&'static self) {
        let global_descriptor_table_register = {
            GlobalDescriptorTableRegister {
                limit: (size_of::<Self>() - 1) as u16,
                base: self as *const _ as usize,
            }
        };
        unsafe {
            asm!(
                "cli",
                "lgdt [{}]",
                in(reg) &global_descriptor_table_register,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}
unsafe impl Send for GlobalDescriptorTableRegister {}
unsafe impl Sync for GlobalDescriptorTableRegister {}
