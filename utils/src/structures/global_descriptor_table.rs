use crate::flag;

#[repr(C, packed(2))]
pub struct GlobalDescriptorTableRegister32 {
    pub limit: u16,
    pub base: *const GlobalDescriptorTable,
}

struct AccessByte(u8);
struct LimitFlags(u8);

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

pub struct GlobalDescriptorTable {
    null: GlobalDescriptorTableEntry32,
    kernel_code: GlobalDescriptorTableEntry32,
    kernel_data: GlobalDescriptorTableEntry32,
}

impl AccessByte {
    /// Start with a zeroed byte.
    pub const fn new() -> Self {
        Self(0)
    }
    // Is this a viable segment?
    flag!(present, 7);
    /// What is the privilege level (DPL).
    pub const fn dpl(mut self, level: u8) -> Self {
        self.0 |= (level & 0x3) << 5;
        self
    }
    // Is this a code or data segment (defaults to system segment).
    flag!(code_or_data, 4);
    // Will this segment constain executable code?.
    flag!(executable, 3);
    // Will the segment grow downwards?
    flag!(direction, 2);
    // Can this code be executed from lower privilege segments.
    flag!(conforming, 2);
    // Can this segment be read or it is only executable?.
    flag!(readable, 1);
    // Is this segment NOT read only?.
    flag!(writable, 1);
}

impl LimitFlags {
    pub const fn new() -> Self {
        Self(0)
    }
    // Toggle on paging for this segment (limit *= 0x1000)
    flag!(paging, 7);
    // Is this segment going to use 32bit memory?
    flag!(protected, 6);
    // Set long mode flag, this will also clear protected mode
    flag!(long, 5);
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

impl GlobalDescriptorTable {
    pub const fn default() -> Self {
        GlobalDescriptorTable {
            null: GlobalDescriptorTableEntry32::new(0, 0, AccessByte::new(), LimitFlags::new()),
            kernel_code: GlobalDescriptorTableEntry32::new(
                0,
                0xfffff,
                AccessByte::new()
                    .set_chain_present()
                    .dpl(0)
                    .set_chain_code_or_data()
                    .set_chain_executable()
                    .set_chain_readable(),
                LimitFlags::new().set_chain_paging().set_chain_protected(),
            ),
            kernel_data: GlobalDescriptorTableEntry32::new(
                0,
                0xfffff,
                AccessByte::new()
                    .set_chain_present()
                    .dpl(0)
                    .set_chain_code_or_data()
                    .set_chain_writable(),
                LimitFlags::new().set_chain_paging().set_chain_protected(),
            ),
        }
    }
}

unsafe impl Send for GlobalDescriptorTableRegister32 {}
unsafe impl Sync for GlobalDescriptorTableRegister32 {}
