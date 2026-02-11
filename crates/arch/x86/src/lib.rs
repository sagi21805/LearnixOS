#![no_std]
#![feature(macro_metavar_expr_concat)]
#![feature(stmt_expr_attributes)]
#![feature(const_trait_impl)]
#![feature(abi_x86_interrupt)]
#![feature(const_default)]
pub mod instructions;
#[cfg(target_arch = "x86_64")]
pub mod memory_map;
pub mod pic8259;
pub mod registers;
pub mod structures;
