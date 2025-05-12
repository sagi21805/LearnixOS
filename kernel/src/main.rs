#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]
#![allow(static_mut_refs)]
#![feature(unsafe_cell_access)]
#![feature(ptr_alignment_type)]

mod allocators;
mod drivers;

use allocators::page_allocator::ALLOCATOR;
use constants::enums::PageSize;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::panic::PanicInfo;
use cpu_utils::registers::{cr3_read, get_current_page_table};
use cpu_utils::structures::paging::page_tables::{PageTable, PageTableEntry};
use drivers::vga_display::color_code::Color;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn _start() -> ! {
    asm!(
        "mov {0}, 0x10",
        "mov ds, {0}",
        "mov es, {0}",
        "mov ss, {0}",
        out(reg) _
    );
    ok_msg!("Entered Protected Mode");
    ok_msg!("Enabled Paging");
    ok_msg!("Entered Long Mode");
    println!("Testing");
    let a: &'static mut PageTable = core::mem::transmute(cr3_read());
    let a_address = a as *const _ as usize;
    let b = &a.entries[3] as *const _ as usize;
    let b_index = ((b as usize) & (0x1000 - 1)) / size_of::<PageTableEntry>();
    let table_address_from_entry = b & (usize::MAX - (0x1000 - 1));
    // let phys_ptr = &IDENTITY_PAGE_TABLE_L4 as *const _ as usize;
    // println!("Paging address: {}", a);
    // println!("Paging address: {}", phys_ptr);
    let a = ALLOCATOR.assume_init_ref().alloc(PageSize::Regular.into());
    println!("{:?}", a);
    // panic!("Test");
    // let a = get_current_page_table().find_available_page(constants::enums::PageSize::Huge);
    println!("address from table {:x?}", a_address);
    println!("entry address {:x?}", b);
    println!("Index in table: {:x?}", b_index ; color = ColorCode::new(Color::Magenta, Color::Blue));
    println!("table address from entry: {:x?}", table_address_from_entry);

    loop {}
}

/// This function is called on panic.
#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
    fail_msg!("{}", _info ; color = ColorCode::new(Color::Yellow, Color::Black));
    loop {}
}
