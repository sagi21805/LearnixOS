#![no_std]
#![feature(macro_metavar_expr_concat)]
#![feature(stmt_expr_attributes)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(iter_map_windows)]

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod instructions;
#[cfg(target_arch = "x86_64")]
pub mod memory_map;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod pic8259;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod registers;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod structures;
