#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _start() -> ! {
    okprintln!("Entered Protected Mode");
    okprintln!("Enabled Paging");
    okprintln!("Entered Long Mode");
    eprintln!("Custom Failure!");
}
