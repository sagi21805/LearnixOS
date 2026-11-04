#![no_std]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(const_trait_impl)]
#![feature(ptr_alignment_type)]
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
pub mod volatile;
