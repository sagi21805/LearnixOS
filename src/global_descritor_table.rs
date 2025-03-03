use core::arch::asm;

#[link_section = ".global_descriptor_table"]
pub static GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTable = GlobalDescriptorTable::default();

#[repr(C)]
pub struct GlobalDescriptorTableRegister32 {
    size: u16,
    offset: u32
}

#[repr(C)]
pub struct GlobalDescriptorTableRegister64 {
    size: u16,
    offset: u64
}

impl GlobalDescriptorTableRegister32 {

    #[link_section = ".second_stage"]
    pub fn new(
        size: u16, 
        offset: *const GlobalDescriptorTable
    ) -> Self {
        Self {
            size: ((size * (size_of::<GlobalDescriptorTableEntry32>()) as u16) - 1) as u16,
            offset: offset as u32
        }
    }
}

#[repr(C)]
pub struct GlobalDescriptorTableEntry32 {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access_byte: u8,
    /// Low 4 bits limit high 4 bits flags
    limit_flags: u8, 
    base_high: u8
}

impl GlobalDescriptorTableEntry32 {

    #[link_section = ".second_stage"]
    pub const fn new(
        base: u32,
        limit: u32,
        access_byte: u8,
        flags: u8,
    ) -> Self {

        let base_low = (base & 0xffff) as u16;
        let base_mid = ((base >> 0xf) & 0xff) as u8;
        let base_high = ((base >> 0x18 ) & 0xff) as u8; 
        let limit_low = (limit & 0xffff) as u16;
        let limit_high = ((limit >> 0xf) & 0xf) as u8;
        let limit_flags = ((flags & 0xff) << 0x4) | limit_high; 
        Self {
            limit_low,
            base_low,
            base_mid,
            access_byte,
            limit_flags,
            base_high
        }
    }

}

pub struct GlobalDescriptorTable {
    null: GlobalDescriptorTableEntry32,
    kernel_code: GlobalDescriptorTableEntry32,
    kernel_data: GlobalDescriptorTableEntry32,
}

impl GlobalDescriptorTable {

    #[link_section = ".second_stage"]
    pub const fn default() -> Self {
        GlobalDescriptorTable {
            null: GlobalDescriptorTableEntry32::new(0, 0, 0, 0),
            kernel_code: GlobalDescriptorTableEntry32::new(0, 0xfffff, 0x9a, 0xc),
            kernel_data: GlobalDescriptorTableEntry32::new(0, 0xfffff, 0x92, 0xc),
        }
    }

    #[link_section = ".second_stage"]
    pub fn load() {
        let gdtr = GlobalDescriptorTableRegister32::new(
            3, &GLOBAL_DESCRIPTOR_TABLE
        );
        unsafe {
            asm!(
                "cli", // disable interrupts
                "lgdt [{0}]",
                in(reg) &gdtr as *const GlobalDescriptorTableRegister32 as u32,
                options(readonly, nostack, preserves_flags)
            )
        }
    }

}