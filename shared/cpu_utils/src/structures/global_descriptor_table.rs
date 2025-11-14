use common::{
    enums::{ProtectionLevel, SystemSegmentType},
    flag,
};

use crate::{instructions, structures::segments::SegmentSelector};

// ANCHOR: access_byte
struct AccessByte(u8);
// ANCHOR_END: access_byte

impl AccessByte {
    // ANCHOR: access_byte_new
    /// Creates an access byte with all flags turned off.
    pub const fn new() -> Self {
        Self(0)
    }
    // ANCHOR_END: access_byte_new

    // ANCHOR: access_byte_present
    // Is this a valid segment?
    // for all active segments this should be turned on.
    flag!(present, 7);
    // ANCHOR_END: access_byte_present

    // ANCHOR: access_byte_privilege_level
    /// Sets the privilege level while returning self.
    /// This is corresponding to the cpu ring of this
    /// segment 0 is commonly called kernel mode, 4 is
    /// commonly called user mode
    pub const fn dpl(mut self, level: ProtectionLevel) -> Self {
        self.0 |= (level as u8) << 5;
        self
    }
    // ANCHOR_END: access_byte_privilege_level

    // ANCHOR: access_byte_type
    /// Set the type for a system segment.
    ///
    /// **Note:** This function is relevant only for system
    /// segments
    pub const fn set_system_type(
        mut self,
        system_type: SystemSegmentType,
    ) -> Self {
        self.0 |= system_type as u8;
        self
    }
    // ANCHOR_END: access_byte_type

    // ANCHOR: access_byte_code_data
    // Is this a code / data segment or a system segment.
    flag!(code_or_data, 4);
    // ANCHOR_END: access_byte_code_data

    // ANCHOR: access_byte_executable
    // Will this segment contains executable code?
    flag!(executable, 3);
    // ANCHOR_END: access_byte_executable

    // ANCHOR: access_byte_direction
    // Will the segment grow downwards?
    // relevant for non executable segments
    flag!(direction, 2);
    // ANCHOR_END: access_byte_direction

    // ANCHOR: access_byte_conforming
    // Can this code be executed from lower privilege
    // segments. relevant to executable segments
    flag!(conforming, 2);
    // ANCHOR_END: access_byte_conforming

    // ANCHOR: access_byte_readable
    // Can this segment be read or it is only executable?
    // relevant for code segment
    flag!(readable, 1);
    // ANCHOR_END: access_byte_readable

    // ANCHOR: access_byte_writable
    // Is this segment writable?
    // relevant for data segments
    flag!(writable, 1);
    // ANCHOR_END: access_byte_writable
}

// ANCHOR: limit_flags
/// Low 4 bits limit high 4 bits flags
struct LimitFlags(u8);
// ANCHOR_END: limit_flags

// ANCHOR: limit_flags_impl
impl LimitFlags {
    // ANCHOR: limit_flags_new
    /// Creates a default limit flags with all flags turned
    /// off.
    pub const fn new() -> Self {
        Self(0)
    }
    // ANCHOR_END: limit_flags_new

    // ANCHOR: limit_flags_granularity
    // Toggle on paging for this segment (limit *= 0x1000)
    flag!(granularity, 7);
    // ANCHOR_END: limit_flags_granularity

    // ANCHOR: limit_flags_protected
    // Is this segment going to use 32bit mode?
    flag!(protected, 6);
    // ANCHOR_END: limit_flags_protected

    // ANCHOR: limit_flags_long
    // Set long mode flag, this will also clear protected
    // mode
    flag!(long, 5);
    // ANCHOR_END: limit_flags_long
}
// ANCHOR_END: limit_flags_impl

// ANCHOR: gdt_entry32
#[repr(C, packed)]
struct GlobalDescriptorTableEntry32 {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access_byte: AccessByte,
    limit_flags: LimitFlags,
    base_high: u8,
}
// ANCHOR_END: gdt_entry32

// ANCHOR: gdt_entry32_impl
impl GlobalDescriptorTableEntry32 {
    /// Construct and empty entry
    // ANCHOR: gdt_entry32_empty
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
    // ANCHOR_END: gdt_entry32_empty

    /// Create a new entry
    ///
    /// # Parameters
    ///
    /// - `base`: The base address of the segment
    /// - `limit`: The size of the segment
    /// - `access_byte`: The type and access privileges of the entry
    /// - `flags`: Configuration flags of the entry
    // ANCHOR: gdt_entry32_new
    pub const fn new(
        base: u32,
        limit: u32,
        access_byte: AccessByte,
        flags: LimitFlags,
    ) -> Self {
        // Split base into the appropriate parts
        let base_low = (base & 0xffff) as u16;
        let base_mid = ((base >> 0x10) & 0xff) as u8;
        let base_high = ((base >> 0x18) & 0xff) as u8;
        // Split limit into the appropriate parts
        let limit_low = (limit & 0xffff) as u16;
        let limit_high = ((limit >> 0x10) & 0xf) as u8;
        // Combine the part of the limit size with the flags
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
    // ANCHOR_END: gdt_entry32_new
}
// ANCHOR_END: gdt_entry32_impl

// ANCHOR: gdtr
#[repr(C, packed)]
pub struct GlobalDescriptorTableRegister {
    pub limit: u16,
    pub base: usize,
}
// ANCHOR_END: gdtr

// ANCHOR: system_segment_descriptor64
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
// ANCHOR_END: system_segment_descriptor64

// ANCHOR: system_segment_descriptor64_impl
impl SystemSegmentDescriptor64 {
    /// Construct an empty system segment
    // ANCHOR: system_segment_descriptor64_empty
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
    // ANCHOR_END: system_segment_descriptor64_empty

    #[cfg(target_arch = "x86_64")]
    /// Construct a new system segment
    ///
    /// # Parameters
    ///
    /// - `base`: The base address of the segment
    /// - `limit`: The limit value of the segment, for each segment this
    ///   may mean something different.
    /// - `segment_type`: The type of the constructed segment
    // ANCHOR: system_segment_descriptor64_new
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
    // ANCHOR_END: system_segment_descriptor64_new
}

// ANCHOR: gdt_protected
/// Initial temporary GDT
#[repr(C, packed)]
pub struct GlobalDescriptorTableProtected {
    null: GlobalDescriptorTableEntry32,
    code: GlobalDescriptorTableEntry32,
    data: GlobalDescriptorTableEntry32,
}
// ANCHOR_END: gdt_protected

// ANCHOR: gdt_protected_impl
impl GlobalDescriptorTableProtected {
    /// Creates default global descriptor table for
    /// protected mode
    // ANCHOR: gdt_default
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
    // ANCHOR_END: gdt_default

    // ANCHOR: gdt_load
    /// Load the GDT with the `lgdt` instruction
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
    // ANCHOR_END: gdt_load
}
// ANCHOR_END: gdt_protected_impl

// ANCHOR: gdt_long
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
// ANCHOR_END: gdt_long

impl GlobalDescriptorTableLong {
    /// Creates default global descriptor table for long
    /// mode
    // ANCHOR: gdt_long_default
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
    // ANCHOR_END: gdt_long_default

    // ANCHOR: gdt_long_load_tss
    /// Load the TSS segment into the GDT
    pub fn load_tss(&mut self, tss: SystemSegmentDescriptor64) {
        self.tss = tss;
        let tss_selector = SegmentSelector::default().set_table_index(5);
        unsafe {
            instructions::ltr(tss_selector);
        }
    }
    // ANCHOR_END: gdt_long_load_tss

    // ANCHOR: gdt_long_load
    /// Load the GDT with the `lgdt` instruction
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
    // ANCHOR_END: gdt_long_load
}
unsafe impl Send for GlobalDescriptorTableRegister {}
unsafe impl Sync for GlobalDescriptorTableRegister {}
