use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Field, Ident, ItemStruct, LitInt, Meta, Path, Token, Type, TypePath,
    Visibility, parse::Parse, parse_quote,
};

mod keyword {
    syn::custom_keyword!(flag);
    syn::custom_keyword!(flag_type);
}

#[repr(u8)]
#[derive(Debug)]
pub enum FlagPermission {
    Read = 1,
    Write = 2,
    ReadWrite = 3,
    Clear(usize) = 4,
    ReadClear(usize) = 5,
    WriteClear(usize) = 6,
    ReadWriteClear(usize) = 7,
}

impl FlagPermission {
    fn tag(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }

    pub fn has_read(&self) -> bool {
        (self.tag() & 0b001) != 0
    }

    pub fn has_write(&self) -> bool {
        (self.tag() & 0b010) != 0
    }

    pub fn has_clear(&self) -> Option<usize> {
        match self {
            FlagPermission::Clear(val)
            | FlagPermission::ReadClear(val)
            | FlagPermission::WriteClear(val)
            | FlagPermission::ReadWriteClear(val) => Some(*val),
            _ => None,
        }
    }
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

#[derive(Debug)]
pub struct FlagType {
    flag_type_token: keyword::flag_type,
    equal: Token![=],
    ty: Box<TypePath>,
}

impl Parse for FlagType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(FlagType {
            flag_type_token: input.parse()?,
            equal: input.parse()?,
            ty: input.parse()?,
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

pub struct Bitflags<'a> {
    struct_name: &'a Ident,
    struct_type: Box<TypePath>,
    fields: Vec<BitField<'a>>,
}

impl<'a> TryFrom<&'a ItemStruct> for Bitflags<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a ItemStruct) -> Result<Self, Self::Error> {
        let mut fields = value
            .fields
            .iter()
            .map(|f| BitField::try_from(f))
            .collect::<syn::Result<Vec<_>>>()?;

        let mut offset = 0;
        for f in &mut fields {
            f.offset = Some(offset);
            offset += f.size;
        }
        let struct_type: TypePath = match offset {
            0..=8 => parse_quote!(u8),
            9..=16 => parse_quote!(u16),
            17..=32 => parse_quote!(u32),
            33..=64 => parse_quote!(u64),
            65..=128 => parse_quote!(u128),
            _ => {
                return Err(syn::Error::new_spanned(
                    &value,
                    "Expected bit to be between 0 - 128",
                ));
            }
        };
        Ok(Bitflags {
            struct_name: &value.ident,
            struct_type: Box::new(struct_type),
            fields,
        })
    }
}

pub struct BitField<'a> {
    permissions: FlagPermission,
    vis: &'a Visibility,
    name: &'a Ident,
    uint_ty: Box<TypePath>,
    additional_ty: Option<Box<TypePath>>,
    size: usize,
    offset: Option<usize>,
}

impl<'a> TryFrom<&'a Field> for BitField<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a Field) -> Result<Self, Self::Error> {
        let (min_uint, size) = get_closest_uint(&value.ty)?;

        let min_uint_ty = Box::new(min_uint);

        if value.attrs.len() == 0 {
            return Ok(BitField {
                permissions: FlagPermission::ReadWrite,
                vis: &value.vis,
                name: value
                    .ident
                    .as_ref()
                    .expect("Fields must have a name"),
                uint_ty: min_uint_ty,
                additional_ty: None,
                size,
                offset: None,
            });
        }

        if value.attrs.len() > 1 {
            return Err(syn::Error::new_spanned(
                value,
                "Fields must have at most one attribute",
            ));
        }

        let attr = &value.attrs[0];
        if let Meta::List(list) = &attr.meta {
            if list.path.get_ident().ok_or(syn::Error::new_spanned(
                list,
                "Meta list must contain single ident path",
            ))? == "flag"
            {
                let FlagAttribute {
                    permissions,
                    flag_type,
                } = syn::parse2::<FlagAttribute>(list.tokens.clone())?;

                Ok(BitField {
                    permissions,
                    vis: &value.vis,
                    name: value
                        .ident
                        .as_ref()
                        .expect("Field must have a name`"),
                    uint_ty: min_uint_ty,
                    additional_ty: flag_type.map(|(_, ftype)| ftype.ty),
                    size,
                    offset: None,
                })
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
        }
    }
}

impl<'a> Bitflags<'a> {
    fn fn_read(&self, field: &'a BitField) -> TokenStream2 {
        let BitField {
            permissions: _,
            vis,
            name,
            uint_ty,
            additional_ty: _,
            size,
            offset,
        } = field;

        let offset =
            offset.expect("Fields not initialized offset not found.");

        let name = format_ident!("get_{}", name);
        let struct_type = &self.struct_type;
        if field.permissions.has_read() {
            quote! {
                #vis fn #name(&self) -> #uint_ty {
                    unsafe {
                        let addr = self as *const _ as *mut #struct_type;
                        let val = std::ptr::read_volatile(addr);
                        (((val >> #offset) & ((1 << #size) - 1))) as #uint_ty
                    }
                }
            }
        } else {
            TokenStream2::new()
        }
    }

    fn fn_write(&self, field: &BitField) -> TokenStream2 {
        let BitField {
            permissions: _,
            vis,
            name,
            uint_ty,
            additional_ty,
            size,
            offset,
        } = field;

        let offset = offset
            .as_ref()
            .expect("Fields not initialized offset not found.");

        let name = format_ident!("set_{}", name);
        let struct_type = &self.struct_type;
        let struct_type = &self.struct_type;

        let ty = if let Some(additional) = additional_ty {
            additional
        } else {
            uint_ty
        };

        if field.permissions.has_write() {
            quote! {
                #vis fn #name(&mut self, v: #ty) {
                    debug_assert!(
                        (v as usize) < 1 << #size,
                        "Size of value is bigger then possible"
                    );
                    unsafe {
                        let addr = self as *const _ as *mut #struct_type;
                        let val = std::ptr::read_volatile(addr);
                        let clear = val & !(((1 << #size) - 1) << #offset);
                        let new = clear | (((v as #struct_type) << #offset) as #struct_type);
                        std::ptr::write_volatile(addr, new);
                    }
                }
            }
        } else {
            TokenStream2::new()
        }
    }

    fn fn_clear(&self, field: &'a BitField) -> TokenStream2 {
        let BitField {
            permissions: _,
            vis,
            name,
            uint_ty: _,
            additional_ty: _,
            size,
            offset,
        } = field;

        let offset =
            offset.expect("Fields not initialized offset not found.");

        let name = format_ident!("clear_{}", name);
        let struct_type = &self.struct_type;

        if let Some(clear_val) = field.permissions.has_clear() {
            quote! {
                #vis fn #name(&mut self) {
                    unsafe {
                        let addr = self as *const _ as *mut #struct_type;
                        let val = core::ptr::read_volatile(addr);
                        let clear = val & !(((1 << #size) - 1) << #offset);
                        let new = clear | (((#clear_val as #struct_type) << #offset) as #struct_type);
                        std::ptr::write_volatile(addr, new);
                    }
                }
            }
        } else {
            TokenStream2::new()
        }
    }

    fn debug_impl(&self) -> TokenStream2 {
        let field_names =
            self.fields.iter().map(|f| f.name).collect::<Vec<_>>();
        let field_get = field_names
            .iter()
            .map(|n| format_ident!("get_{}", n))
            .collect::<Vec<_>>();

        let struct_name = self.struct_name;
        quote! {
            impl std::fmt::Debug for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!(#struct_name))
                        #(.field(stringify!(#field_names), &self.#field_get()))*
                        .finish()
                }
            }
        }
    }
}

// TODO REMOVE NESTING
fn get_closest_uint(ty: &Type) -> syn::Result<(TypePath, usize)> {
    if let Type::Path(ty) = ty {
        let ident = ty.path.get_ident().ok_or(syn::Error::new_spanned(
            ty,
            "Expected single ident type",
        ))?;
        let type_name = ident.to_string();

        if let Some(bit_str) = type_name.strip_prefix('B') {
            if let Ok(bits) = bit_str.parse::<usize>() {
                match bits {
                    0..=8 => Ok((parse_quote!(u8), bits)),
                    9..=16 => Ok((parse_quote!(u16), bits)),
                    17..=32 => Ok((parse_quote!(u32), bits)),
                    33..=64 => Ok((parse_quote!(u64), bits)),
                    65..=128 => Ok((parse_quote!(u128), bits)),
                    _ => Err(syn::Error::new_spanned(
                        ty,
                        "Expected bit to be between 0 - 128",
                    )),
                }
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
            ty,
            "Expected Type to be single ident path",
        ))
    }
}
pub fn bitfields_impl(s: ItemStruct) -> syn::Result<TokenStream2> {
    let bitfield = Bitflags::try_from(&s)?;
    let min_uint = &bitfield.struct_type;
    let vis = &s.vis;
    let ident = &s.ident;

    let functions = bitfield
        .fields
        .iter()
        .map(|b| {
            vec![
                bitfield.fn_read(b),
                bitfield.fn_write(b),
                // bitfield.fn_clear(b),
            ]
        })
        .collect::<Vec<Vec<TokenStream2>>>();

    let debug_impl = bitfield.debug_impl();

    let struct_def = quote! {
        #vis struct #ident ( #min_uint );

        impl #ident {

            pub fn new() -> Self {
                Self(0)
            }

            #( #(#functions)* )*

        }

        #debug_impl

    };

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
