#![no_std]
#![feature(ptr_alignment_type)]
#![feature(macro_metavar_expr_concat)]

#[macro_use]
pub mod macros;
pub mod address_types;
pub mod bitmap;
pub mod constants;
pub mod enums;
pub mod error;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod vga_display;
