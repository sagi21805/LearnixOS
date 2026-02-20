use std::fmt::Debug;

use macros::bitfields;

fn main() {
    println!("Hello, world!");

    let mut f = MyFlags::new();
    f.set_a(t::Test::Two);
    f.set_d(3);
    f.set_c(3);
    println!("{:#?}", f);
}
mod t {

    #[repr(u8)]
    #[derive(Clone, Copy)]
    pub enum Test {
        One = 1,
        Two = 2,
    }
}

#[bitfields]
pub struct MyFlags {
    #[flag(rw, flag_type = t::Test)]
    a: B3,
    b: B3,
    c: B3,
    d: B9,
}
