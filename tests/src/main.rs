#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(never_type)]

use macros::bitfields;

mod book;
mod test;
use test::{Nested, Test};

fn main() {
    println!("Hello, world!");

    let mut f = MyFlags::new();
    let nested = Nested::new().a().b(Test::SomeRandomName);
    f.set_d(true);
    f.set_c(true);
    f.set_d(false);
    println!("{:#x?}", f);

    let x: u8 = 242;
}

#[bitfields]
pub struct MyFlags {
    a: B1,
    b: B1,
    c: B1,
    d: B1,
}
