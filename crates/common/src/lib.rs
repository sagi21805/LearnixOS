#![no_std]
#![deny(clippy::all)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![feature(ptr_alignment_type)]
#![feature(const_default)]
#![feature(pattern_types, pattern_type_macro)]
#![expect(
    internal_features,
    reason = "`PageIndex` is a pattern type, which is an internal feature"
)]

pub mod address_types;
pub mod bitmap;
pub mod constants;
pub mod enums;
pub mod error;
pub mod iter;
pub mod page_index;
#[cfg(target_arch = "x86_64")]
pub mod ring_buffer;
pub mod volatile;
