#![feature(const_trait_impl)]

use proc_macro::TokenStream;
mod book;

fn main() {
    println!("Hello, world!");
}

#[proc_macro]
pub fn custom_proc_macro(input: TokenStream) -> TokenStream {
    eprintln!("{:?}", input);
    input
}
