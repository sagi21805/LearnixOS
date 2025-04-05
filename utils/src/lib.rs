#![no_std]
#![feature(macro_metavar_expr_concat)]
#![feature(adt_const_params)]
#![feature(ptr_alignment_type)]
#![feature(unsafe_cell_access)]


#[cfg(feature = "screen")]
pub mod screen;

pub mod structures;
