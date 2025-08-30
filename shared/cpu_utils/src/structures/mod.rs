pub mod global_descriptor_table;
#[cfg(target_arch = "x86_64")]
pub mod interrupt_descriptor_table;
pub mod mbr;
pub mod paging;
pub mod segments;
