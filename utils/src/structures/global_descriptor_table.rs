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
    /// Is this a viable segment?.
    pub const fn present(mut self) -> Self {
        self.0 |= 0x80;
        self
    }
    /// What is the privilege level (DPL).
    pub const fn dpl(mut self, level: u8) -> Self {
        self.0 |= (level & 0x3) << 5;
        self
    }
    /// Is this a code or data segment (defaults to system segment).
    pub const fn code_or_data(mut self) -> Self {
        self.0 |= 0x10;
        self
    }
    /// Will this segment contain executable code?.
    pub const fn executable(mut self) -> Self {
        self.0 |= 0x08;
        self
    }
    /// Will the segment grow downwards?
    pub const fn direction(mut self) -> Self {
        self.0 |= 0x04;
        self
    }
    /// Can this code be executed from lower privilege segments.
    pub const fn conforming(mut self) -> Self {
        self.0 |= 0x04;
        self
    }
    /// Can this segment be read or it is only executable?.
    pub const fn readable(mut self) -> Self {
        self.0 |= 0x02;
        self
    }
    /// Is this segment NOT read only?.
    pub const fn writable(mut self) -> Self {
        self.0 |= 0x02;
        self
    }
}

impl LimitFlags {
    pub const fn new() -> Self {
        Self(0)
    }
    /// Toggle on paging for this segment (limit *= 0x1000)
    pub const fn paging(mut self) -> Self {
        self.0 |= 0x80;
        self
    }
    /// Is this segment going to use 32bit memory?
    pub const fn protected(mut self) -> Self {
        self.0 |= 0x40;
        self
    }
    /// Set long mode flag, this will also clear protected mode
    pub const fn long(mut self) -> Self {
        self.0 &= 0xA0;
        self
    }
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
                    .present()
                    .dpl(0)
                    .code_or_data()
                    .executable()
                    .readable(),
                LimitFlags::new().paging().protected(),
            ),
            kernel_data: GlobalDescriptorTableEntry32::new(
                0,
                0xfffff,
                AccessByte::new().present().dpl(0).code_or_data().writable(),
                LimitFlags::new().paging().protected(),
            ),
        }
    }
}

unsafe impl Send for GlobalDescriptorTableRegister32 {}
unsafe impl Sync for GlobalDescriptorTableRegister32 {}
