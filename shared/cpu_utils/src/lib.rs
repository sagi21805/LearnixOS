#![no_std]
#![feature(macro_metavar_expr_concat)]
#![feature(adt_const_params)]
#![feature(ptr_alignment_type)]
#![feature(unsafe_cell_access)]
#![feature(abi_x86_interrupt)]

#[cfg(target_arch = "x86_64")]
pub mod instructions;
pub mod registers;
pub mod structures;
