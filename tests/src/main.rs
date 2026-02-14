use macros::bitfields;

fn main() {
    println!("Hello, world!");
}

#[bitfields]
pub struct MyFlags {
    a: u32,
    b: u32,
    c: u32,
}
