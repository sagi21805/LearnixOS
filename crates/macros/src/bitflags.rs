use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    AttrStyle, Attribute, Block, Ident, ItemStruct, LitInt, Meta, Path,
    ReturnType, Token,
    parse::{Parse, Peek},
    token::Paren,
};

pub struct BitFields {
    s: ItemStruct,
}

impl ToTokens for BitFields {
    fn to_tokens(&self, tokens: &mut TokenStream2) {}
}

#[derive(Debug)]
pub enum FlagPermission {
    Read,
    Write,
    ReadWrite,
    Clear(usize),
    ReadClear(usize),
    WriteClear(usize),
    ReadWriteClear(usize),
}

impl Default for FlagPermission {
    fn default() -> Self {
        FlagPermission::ReadWrite
    }
}

impl Parse for FlagPermission {
    /// Parse flag permission of R, W and C(<lit_int>) for a member of this
    /// enum.
    ///
    /// The input may be, R, RW, C(<lit_int>), RWC(<lit_int>),
    /// RC(<lit_int>), WC(<lit_int>)
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let permission = input.parse::<Ident>()?.to_string();

        let rw = match (permission.contains('r'), permission.contains('w'))
        {
            (true, false) => Some(FlagPermission::Read),
            (false, true) => Some(FlagPermission::Write),
            (true, true) => Some(FlagPermission::ReadWrite),
            (false, false) => None,
        };

        if permission.contains('c') {
            let content;
            let _ = syn::parenthesized!(content in input);
            let int =
                content.parse::<LitInt>()?.base10_parse::<usize>()?;

            if let Some(rw) = &rw {
                match rw {
                    FlagPermission::Read => {
                        Ok(FlagPermission::ReadClear(int))
                    }
                    FlagPermission::Write => {
                        Ok(FlagPermission::WriteClear(int))
                    }
                    FlagPermission::ReadWrite => {
                        Ok(FlagPermission::ReadWriteClear(int))
                    }
                    _ => unreachable!(),
                }
            } else {
                Ok(FlagPermission::Clear(int))
            }
        } else {
            rw.ok_or(input.error(
                "A flag was not specified at all. Please specify a \
                 combination of R, W, or C(<val>)",
            ))
        }
    }
}

mod keyword {
    syn::custom_keyword!(flag);
    syn::custom_keyword!(flag_type);
}

#[derive(Debug)]
pub struct FlagType {
    flag_type_token: keyword::flag_type,
    equal: Token![=],
    type_: Path,
}

impl Parse for FlagType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(FlagType {
            flag_type_token: input.parse()?,
            equal: input.parse()?,
            type_: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct FlagAttribute {
    permissions: FlagPermission,
    flag_type: Option<(Token![,], FlagType)>,
}

impl Parse for FlagAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let permissions = input.parse()?;
        let flag_type = if input.peek(Token![,]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        Ok(FlagAttribute {
            permissions,
            flag_type,
        })
    }
}

pub fn bitfields_impl(s: ItemStruct) -> syn::Result<TokenStream2> {
    let min_uint = quote! { u8 };
    // let mut read_fn_names: Vec<Ident> = Vec::new();
    // let mut return_types: Vec<ReturnType> = Vec::new();
    // let mut write_fn_names: Vec<Ident> = Vec::new();
    // let mut read_bodies: Vec<Block> = Vec::new();
    // let mut write_bodies: Vec<Block> = Vec::new();

    // let mut size = 0;

    for f in &s.fields {
        if f.attrs.len() == 0 {
            // todo!();
            continue;
        }

        if f.attrs.len() > 1 {
            return Err(syn::Error::new_spanned(
                f,
                "Fields must have at most one attribute",
            ));
        }

        let attr = &f.attrs[0];
        let flag_attr = if let Meta::List(list) = &attr.meta {
            if list.path.get_ident().ok_or(syn::Error::new_spanned(
                list,
                "Meta list must contain single ident path",
            ))? == "flag"
            {
                Ok(syn::parse2::<FlagAttribute>(list.tokens.clone())?)
            } else {
                Err(syn::Error::new_spanned(
                    list,
                    "Attribute on bitfields struct should only include \
                     flag",
                ))
            }
        } else {
            Err(syn::Error::new_spanned(
                &attr.meta,
                "Attribute on bitfields struct should only include \
                 flag(permission, flag_type=type)",
            ))
        }?;

        println!("Attr: {:?}", flag_attr)
    }

    let vis = &s.vis;
    let ident = &s.ident;
    let struct_def = quote! {
        #vis struct #ident ( #min_uint );
    };
    //     #(
    //         pub fn #write_fn_names() {

    //         }
    //     )*

    //     #(
    //         pub fn #read_fn_names() {

    //         }
    //     )*
    // };

    // #(
    //     pub fn #fn_names(#(#args)*) #return_types {
    //         #bodies
    //     }
    // )*
    return Ok(struct_def);
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
                #[flag(rwc(40), flag_type = Some::Type)]
                a: u32,

                #[flag(rw)]
                b: u32,

                #[flag(rw)]
                c: u32
            }

        };

        let input = syn::parse2(example).unwrap();
        let output_tokens = bitfields_impl(input).unwrap();
        println!("{:#?}", output_tokens);
    }
}
