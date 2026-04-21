use core::fmt;
use std::panic::Location;

fn main() {
    panic!("This is a custom message");
}

pub struct PanicInfo<'a> {
    message: &'a fmt::Arguments<'a>,
    location: &'a Location<'a>,
    can_unwind: bool,
    force_no_backtrace: bool,
}
