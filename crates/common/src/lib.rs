#![no_std]
#![deny(clippy::all)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![feature(ptr_alignment_type)]
#![feature(macro_metavar_expr_concat)]
#![feature(const_slice_make_iter)]
#![feature(const_default)]

#[macro_use]
pub mod macros;
pub mod address_types;
pub mod bitmap;
pub mod constants;
pub mod enums;
pub mod error;
pub mod iter;
pub mod late_init;
pub mod ring_buffer;
pub mod volatile;
