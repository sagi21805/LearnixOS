#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![allow(dead_code)]
#![feature(optimize_attribute)]
#![feature(ptr_as_ref_unchecked)]

#[no_mangle]
#[link_section = ".second_stage_entry"]
#[cfg(feature = "32bit")]
pub extern "C" fn second_stage_entry(mbr: &mut MasterBootRecord) {
    MinimalWriter::print("Entering Second Stage");
}


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // print(_info.message().as_str().unwrap());
    loop {}
}