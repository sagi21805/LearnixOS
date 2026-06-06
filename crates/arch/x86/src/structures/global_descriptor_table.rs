use common::enums::{
    ProtectionLevel, Sections, SegmentDescriptorType, SystemSegmentType,
};
use macros::bitfields;

use crate::{instructions, structures::segments::SegmentSelector};

#[bitfields]
pub struct AccessByte {
    #[flag(r)]
    accessed: B1,
    readable_writable: B1,
    direction_conforming: B1,
    executable: B1,
    #[flag(flag_type = SegmentDescriptorType)]
    segment_type: B1,
    #[flag(flag_type = ProtectionLevel)]
    dpl: B2,
    present: B1,
}

/// Low 4 bits limit high 4 bits flags
#[bitfields]
struct LimitFlags {
    limit_high: B4,
    #[flag(r)]
    reserved: B1,
    long: B1,
    protected: B1,
    granularity: B1,
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
        // Split base into the appropriate parts
        let base_low = (base & 0xffff) as u16;
        let base_mid = ((base >> 0x10) & 0xff) as u8;
        let base_high = ((base >> 0x18) & 0xff) as u8;
        // Split limit into the appropriate parts
        let limit_low = (limit & 0xffff) as u16;
        let limit_high = ((limit >> 0x10) & 0xf) as u8;
        // Combine the part of the limit size with the flags
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

#[bitfields]
pub struct SystemAccessByte {
    #[flag(flag_type = SystemSegmentType)]
    segment_type: B4,
    #[flag(rc(0))]
    zero: B1,
    #[flag(flag_type = ProtectionLevel)]
    dpl: B2,
    present: B1,
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
    #[cfg(target_arch = "x86_64")]
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
    // ANCHOR_END: system_segment_descriptor64_new
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
    #[cfg(target_arch = "x86_64")]
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
