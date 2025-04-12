#[cfg(feature = "global_descriptor_table")]
pub mod global_descriptor_table;

#[cfg(feature = "interrupt_descriptor_table")]
pub mod interrupt_descriptor_table;

pub mod macros;

#[cfg(feature = "master_boot_record")]
pub mod mbr;

#[cfg(feature = "paging")]
pub mod paging;