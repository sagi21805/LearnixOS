#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

use macros::bitfields;

mod buddy;
mod test;
use test::{Nested, Test};

fn main() {
    println!("Hello, world!");

    let mut f = MyFlags::new();
    let nested = Nested::new().a(true).b(Test::SomeRandomName);
    f.set_d(true);
    f.set_c(true);
    println!("{:#x?}", f);
    println!("{:b}", f.0)
}

#[bitfields]
pub struct MyFlags {
    a: B1,
    b: B1,
    c: B1,
    d: B1,
}
