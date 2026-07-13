#![no_std]
#![deny(clippy::all)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![feature(ptr_alignment_type)]
#![feature(const_default)]

pub mod address_types;
#[cfg(target_arch = "x86_64")]
pub mod alloc;
pub mod bitmap;
pub mod constants;
pub mod enums;
pub mod error;
pub mod iter;
pub mod late_init;
#[cfg(target_arch = "x86_64")]
pub mod ring_buffer;
pub mod volatile;
