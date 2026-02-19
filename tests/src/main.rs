use std::fmt::Debug;

use macros::bitfields;

fn main() {
    println!("Hello, world!");

    let mut f = MyFlags(0);
    f.set_a(t::Test::Two);
    println!("{:#?}", f);
}
mod t {

    #[repr(u8)]
    #[derive(Clone, Copy, Debug)]
    pub enum Test {
        One = 1,
        Two = 2,
    }
}

#[bitfields]
pub struct MyFlags {
    #[flag(rw, flag_type = t::Test)]
    pub a: B3,
    pub b: B3,
    pub c: B3,
    pub d: B9,
}

impl Debug for MyFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyFlags").field("a", &self.get_a()).finish()
    }
}
