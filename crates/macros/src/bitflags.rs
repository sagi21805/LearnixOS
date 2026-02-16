use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    AttrStyle, Attribute, Block, Field, Ident, ItemStruct, LitInt, Meta,
    Path, ReturnType, Token, Type,
    parse::{Parse, Peek},
    parse_quote,
    token::{Paren, Token},
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

pub struct BitflagField<'a> {
    attr: FlagAttribute,
    field: &'a Field,
}

impl<'a> TryFrom<&'a Field> for BitflagField<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a Field) -> Result<Self, Self::Error> {
        if value.attrs.len() == 0 {
            return Ok(BitflagField {
                attr: FlagAttribute {
                    permissions: FlagPermission::ReadWrite,
                    flag_type: None,
                },
                field: value,
            });
        }

        if value.attrs.len() > 1 {
            return Err(syn::Error::new_spanned(
                value,
                "Fields must have at most one attribute",
            ));
        }

        let attr = &value.attrs[0];
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

        Ok(BitflagField {
            attr: flag_attr,
            field: value,
        })
    }
}

impl<'a> BitflagField<'a> {
    // fn into_functions(min_uint: Type) -> syn::Result<TokenStream2> {}

    fn gen_read_function(
        &self,
        offset: usize,
        struct_name: &Ident,
    ) -> syn::Result<TokenStream2> {
        match self.attr.permissions {
            FlagPermission::Read
            | FlagPermission::ReadWrite
            | FlagPermission::ReadClear(_)
            | FlagPermission::ReadWriteClear(_) => {
                let name = format_ident!(
                    "get_{}",
                    self.field.ident.as_ref().unwrap()
                );
                let return_type = self.get_closest_uint()?;
                Ok(quote! {
                    impl #struct_name {
                        pub fn #name(&self) -> #return_type {
                            self.0 | (1 << #offset)
                        }
                    }
                })
            }
            _ => Err(syn::Error::new_spanned(
                &self.field.ty,
                "Flag attribute does not contain read permission",
            )),
        }
    }

    fn next_offset(&self) -> syn::Result<usize> {
        if let Type::Path(p) = &self.field.ty {
            let ident = p
                .path
                .get_ident()
                .ok_or(syn::Error::new_spanned(
                    p,
                    "Expected type to be single ident",
                ))?
                .to_string();

            Ok(ident[1..].parse::<usize>().map_err(|_| {
                syn::Error::new_spanned(p, "Failed to parse into from num")
            })?)
        } else {
            Err(syn::Error::new_spanned(
                &self.field.ty,
                "Type is not path",
            ))
        }
    }
    // fn gen_write_function(&self) -> syn::Result<TokenStream2> {}

    // fn gen_clear_function(&self) -> syn::Result<TokenStream2> {}

    fn get_closest_uint(&self) -> syn::Result<Type> {
        if let Type::Path(type_path) = &self.field.ty {
            let ident = type_path.path.get_ident().ok_or(
                syn::Error::new_spanned(
                    &self.field.ty,
                    "Expected single ident type",
                ),
            )?;
            let type_name = ident.to_string();

            if type_name.starts_with('B') {
                let bit_str = &type_name[1..];
                if let Ok(bits) = bit_str.parse::<u8>() {
                    return Ok(match bits {
                        0..=8 => parse_quote!(u8),
                        9..=16 => parse_quote!(u16),
                        17..=32 => parse_quote!(u32),
                        33..=64 => parse_quote!(u64),
                        65..=128 => parse_quote!(u128),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &self.field.ty,
                                "Expected bit to be between 0 - 128",
                            ));
                        }
                    });
                } else {
                    Err(syn::Error::new_spanned(
                        ident,
                        "Cannot parse int from type",
                    ))
                }
            } else {
                Err(syn::Error::new_spanned(
                    ident,
                    "Expected type to start with a B",
                ))
            }
        } else {
            Err(syn::Error::new_spanned(
                &self.field.ty,
                format_args!(
                    "Expected type to be a single ident path. Found: {:?}",
                    &self.field.ty
                ),
            ))
        }
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
    let vis = &s.vis;
    let ident = &s.ident;
    let mut struct_def = quote! {
        #vis struct #ident ( #min_uint );
    };

    for f in &s.fields {
        let bitflag = BitflagField::try_from(f)?;

        let read = bitflag.gen_read_function(0, ident);
        struct_def.extend(read.unwrap());
    }

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
                a: B3,

                #[flag(rw)]
                b: B1,

                #[flag(rw)]
                c: B1
            }

        };

        let input = syn::parse2(example).unwrap();
        let output_tokens = bitfields_impl(input).unwrap();
        println!("{:#?}", output_tokens);
    }
}
