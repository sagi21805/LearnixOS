use common::{enums::ProtectionLevel, flag};

#[derive(Clone, Debug, Copy)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn set_rpl(mut self, rpl: ProtectionLevel) -> Self {
        self.0 |= rpl as u16;
        self
    }

    // Use the local descriptor table instead of the global descriptor table
    flag!(use_local_descriptor_table, 2);

    /// Set the index in the table
    ///
    /// **Note:** If a system segment is in the table, it should be counted as occupying two indices
    pub const fn set_table_index(mut self, index: u16) -> Self {
        self.0 |= index << 3;
        self
    }

    pub const fn as_u16(&self) -> u16 {
        self.0
    }

    pub const fn kernel_code() -> Self {
        Self(0x8)
    }
    pub const fn user_code() -> Self {
        Self(0x1b)
    }
}

#[repr(C, packed)]
pub struct TaskStateSegment {
    _reserved0: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    _reserved1: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    _reserved2: u64,
    _reserved3: u16,
    /// An offset from the base address of this struct to the I/O map
    io_map_offset: u16,
}

impl TaskStateSegment {
    pub const fn iomb(&self) -> u16 {
        self.io_map_offset
    }

    pub const fn new() -> Self {
        Self {
            _reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            _reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            _reserved2: 0,
            _reserved3: 0,
            io_map_offset: 0xffff,
        }
    }
}
