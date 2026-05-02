use common::{
    address_types::{CommonAddressFunctions, VirtualAddress},
    enums::{ProtectionLevel, Sections},
};

use macros::bitfields;

#[bitfields]
pub struct SegmentSelector {
    #[flag(flag_type = ProtectionLevel)]
    pub rpl: B2,
    pub use_ldt: B1,
    #[flag(flag_type = Sections)]
    pub section: B13,
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
