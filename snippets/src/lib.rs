#![feature(const_trait_impl)]

use proc_macro::TokenStream;
use syn::Ident;
mod book;

fn main() {
    println!("Hello, world!");
}

#[proc_macro]
pub fn custom_proc_macro(input: TokenStream) -> TokenStream {
    eprintln!("{:?}", input);
    input
}

#[proc_macro]
pub fn foo(_item: TokenStream) -> TokenStream {
    "fn bar() -> u32 { 42 }".parse().unwrap()
}

#[proc_macro_derive(WithHelperAttr, attributes(helper))]
pub fn derive_with_helper_attr(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

// The `_attr` parameter is the attribute's input variables, and the `item`
// parameter is the item the attribute is applied to.
#[proc_macro_attribute]
pub fn return_as_is(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn change_name(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_fn = syn::parse_macro_input!(input as syn::ItemFn);

    item_fn.sig.ident = Ident::new(
        &format!("with_change_{}", item_fn.sig.ident),
        item_fn.sig.ident.span(),
    );

    quote::quote! { #item_fn }.into()
}
