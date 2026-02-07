#![no_std]
#![deny(clippy::all)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(macro_metavar_expr_concat)]
#![feature(adt_const_params)]
#![feature(ptr_alignment_type)]
#![feature(unsafe_cell_access)]
#![feature(abi_x86_interrupt)]

pub mod instructions;
pub mod registers;
pub mod structures;
