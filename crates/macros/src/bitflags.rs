use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{AttrStyle, Block, Ident, ItemStruct, ReturnType};

pub fn bitfields_impl(s: ItemStruct) -> TokenStream2 {
    let min_uint = quote! { u8 };
    let mut read_fn_names: Vec<Ident> = Vec::new();
    let mut return_types: Vec<ReturnType> = Vec::new();
    let mut write_fn_names: Vec<Ident> = Vec::new();
    let mut read_bodies: Vec<Block> = Vec::new();
    let mut write_bodies: Vec<Block> = Vec::new();

    let mut size = 0;

    for f in &s.fields {
        for a in &f.attrs {
            match a.style {
                AttrStyle::Inner(_) => {}
                AttrStyle::Outer => {
                    if let Some(ident) = a.meta.path().get_ident() {
                        if ident.to_string() == "write" {
                            write_fn_names.push(format_ident!(
                                "set_{}",
                                f.ident.clone().unwrap()
                            ));
                        } else {
                            return syn::Error::new_spanned(
                                a,
                                "Expected only write attribute on types",
                            )
                            .to_compile_error()
                            .into();
                        }
                    } else {
                        read_fn_names.push(f.ident.clone().unwrap());
                    }
                }
            }
        }
        if f.attrs.is_empty() {
            read_fn_names.push(f.ident.clone().unwrap());
        }
    }
    let vis = &s.vis;
    let ident = &s.ident;
    let struct_def = quote! {
        #vis struct #ident ( #min_uint );

        #(
            pub fn #write_fn_names() {

            }
        )*

        #(
            pub fn #read_fn_names() {

            }
        )*
    };

    // #(
    //     pub fn #fn_names(#(#args)*) #return_types {
    //         #bodies
    //     }
    // )*
    return struct_def;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitfields_impl;

    #[test]
    fn test_macro() {
        let example = quote! {

            #[bitfields]
            pub struct MyFlags {
                a: u32,
                mut(crate) b: u32,
                c: u32
            }

        };

        let input = syn::parse2(TokenStream2::from(example)).unwrap();
        let output_tokens = bitfields_impl(input);
        println!("{:#?}", output_tokens);
    }
}
