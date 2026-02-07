use common::{
    address_types::VirtualAddress,
    enums::{ProtectionLevel, Sections},
};
use learnix_macros::flag;

#[derive(Clone, Debug, Copy)]
pub struct SegmentSelector(u16);

impl const Default for SegmentSelector {
    /// Default NULL segment selector
    fn default() -> Self {
        Self(0)
    }
}

impl SegmentSelector {
    /// Set the requested privilege level of this selector
    pub const fn set_rpl(mut self, rpl: ProtectionLevel) -> Self {
        self.0 |= rpl as u16;
        self
    }

    // Use the local descriptor table instead of the global
    // descriptor table
    flag!(local_descriptor_table, 2);

    /// Set the index in the table
    ///
    /// **Note:** If a system segment is in the table, it
    /// should be counted as occupying two indices
    pub const fn set_table_index(mut self, index: u16) -> Self {
        self.0 |= index << 3;
        self
    }

    /// Return the underlying u16
    pub const fn as_u16(&self) -> u16 {
        self.0
    }

    /// Default kernel code selector
    pub const fn kernel_code() -> Self {
        Self(Sections::KernelCode as u16)
    }

    /// Default user code selector
    pub const fn user_code() -> Self {
        Self(Sections::UserCode as u16 | ProtectionLevel::Ring3 as u16)
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
            priv_stack_ptr: [unsafe { VirtualAddress::new_unchecked(0) };
                3],
            int_stack_table: [unsafe { VirtualAddress::new_unchecked(0) };
                7],
            io_map_offset: size_of::<Self>() as u16,
        }
    }
}
