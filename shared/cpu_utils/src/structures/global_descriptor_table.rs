use common::flag;
use core::arch::asm;
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
    pub const fn dpl(mut self, level: u8) -> Self {
        self.0 |= (level & 0x3) << 5;
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

#[repr(C, packed(2))]
pub struct GlobalDescriptorTableRegister32 {
    pub limit: u16,
    pub base: *const GlobalDescriptorTable,
}

#[allow(dead_code)]
pub struct GlobalDescriptorTable {
    null: GlobalDescriptorTableEntry32,
    code: GlobalDescriptorTableEntry32,
    data: GlobalDescriptorTableEntry32,
}

impl GlobalDescriptorTable {
    /// Creates default global descriptor table for protected mode
    pub const fn protected_mode() -> Self {
        GlobalDescriptorTable {
            null: GlobalDescriptorTableEntry32::new(0, 0, AccessByte::new(), LimitFlags::new()),
            code: GlobalDescriptorTableEntry32::new(
                0,
                0xfffff,
                AccessByte::new()
                    .present()
                    .dpl(0)
                    .code_or_data()
                    .executable()
                    .readable(),
                LimitFlags::new().granularity().protected(),
            ),
            data: GlobalDescriptorTableEntry32::new(
                0,
                0xfffff,
                AccessByte::new().present().dpl(0).code_or_data().writable(),
                LimitFlags::new().granularity().protected(),
            ),
        }
    }

    /// Creates default global descriptor table for long mode
    pub const fn long_mode() -> Self {
        GlobalDescriptorTable {
            null: GlobalDescriptorTableEntry32::new(0, 0, AccessByte::new(), LimitFlags::new()),
            code: GlobalDescriptorTableEntry32::new(
                0,
                0,
                AccessByte::new()
                    .code_or_data()
                    .present()
                    .writable()
                    .executable(),
                LimitFlags::new().long(),
            ),
            data: GlobalDescriptorTableEntry32::new(
                0,
                0,
                AccessByte::new().code_or_data().present().writable(),
                LimitFlags::new(),
            ),
        }
    }

    pub unsafe fn load(&'static self) {
        let global_descriptor_table_register: GlobalDescriptorTableRegister32 = {
            GlobalDescriptorTableRegister32 {
                limit: (size_of::<GlobalDescriptorTable>() - 1) as u16,
                base: self as *const GlobalDescriptorTable,
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

unsafe impl Send for GlobalDescriptorTableRegister32 {}
unsafe impl Sync for GlobalDescriptorTableRegister32 {}
