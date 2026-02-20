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

/// Permission flags for a bitfield member.
///
/// The enum discriminant encodes R/W/C as a 3-bit mask:
///   bit 0 → Read, bit 1 → Write, bit 2 → Clear
#[repr(u8)]
#[derive(Debug, Default)]
pub enum FlagPermission {
    Read = 1,
    Write = 2,
    #[default]
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

impl Parse for FlagPermission {
    /// Parse flag permissions from a combination of `R`, `W`, and
    /// `C(<lit_int>)`.
    ///
    /// Valid inputs: `r`, `w`, `rw`, `c(<n>)`, `rc(<n>)`, `wc(<n>)`,
    /// `rwc(<n>)` (case-insensitive).
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let permission =
            input.parse::<Ident>()?.to_string().to_lowercase();

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

            match rw {
                Some(FlagPermission::Read) => {
                    Ok(FlagPermission::ReadClear(int))
                }
                Some(FlagPermission::Write) => {
                    Ok(FlagPermission::WriteClear(int))
                }
                Some(FlagPermission::ReadWrite) => {
                    Ok(FlagPermission::ReadWriteClear(int))
                }
                None => Ok(FlagPermission::Clear(int)),
                _ => unreachable!(),
            }
        } else {
            rw.ok_or_else(|| {
                input.error(
                    "No valid flag specified. Please use a combination \
                     of R, W, or C(<val>).",
                )
            })
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

pub struct BitField<'a> {
    permissions: FlagPermission,
    vis: &'a Visibility,
    name: &'a Ident,
    /// Smallest unsigned integer type that can hold `size` bits.
    uint_ty: Box<TypePath>,
    /// Optional user-supplied type for setter arguments (e.g. a newtype).
    additional_ty: Option<Box<TypePath>>,
    size: usize,
    offset: Option<usize>,
}

impl<'a> TryFrom<&'a Field> for BitField<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a Field) -> Result<Self, Self::Error> {
        let (min_uint, size) = get_closest_uint(&value.ty)?;
        let name = value.ident.as_ref().expect("Fields must have a name");

        // No attribute → default ReadWrite permissions.
        if value.attrs.is_empty() {
            return Ok(BitField {
                permissions: FlagPermission::ReadWrite,
                vis: &value.vis,
                name,
                uint_ty: Box::new(min_uint),
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
            let attr_ident = list.path.get_ident().ok_or_else(|| {
                syn::Error::new_spanned(
                    list,
                    "Attribute path must be a single identifier",
                )
            })?;

            if attr_ident != "flag" {
                return Err(syn::Error::new_spanned(
                    list,
                    "Only the `flag` attribute is supported on bitfield \
                     members",
                ));
            }

            let FlagAttribute {
                permissions,
                flag_type,
            } = syn::parse2::<FlagAttribute>(list.tokens.clone())?;

            Ok(BitField {
                permissions,
                vis: &value.vis,
                name,
                uint_ty: Box::new(min_uint),
                additional_ty: flag_type.map(|(_, ft)| ft.ty),
                size,
                offset: None,
            })
        } else {
            Err(syn::Error::new_spanned(
                &attr.meta,
                "Attribute must be in the form `flag(permission)` or \
                 `flag(permission, flag_type = Type)`",
            ))
        }
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
            .map(BitField::try_from)
            .collect::<syn::Result<Vec<_>>>()?;

        let mut offset = 0usize;
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
                    value,
                    "Total bit width must be between 0 and 128",
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


impl<'a> Bitflags<'a> {
    fn fn_read(&self, field: &'a BitField) -> TokenStream2 {
        if !field.permissions.has_read() {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            uint_ty,
            size,
            offset,
            ..
        } = field;
        let offset =
            offset.expect("offset must be set before code generation");
        let fn_name = format_ident!("get_{}", name);
        let struct_type = &self.struct_type;

        quote! {
            #vis fn #fn_name(&self) -> #uint_ty {
                unsafe {
                    let addr = self as *const _ as *mut #struct_type;
                    let val = std::ptr::read_volatile(addr);
                    ((val >> #offset) & ((1 << #size) - 1)) as #uint_ty
                }
            }
        }
    }

    fn fn_write(&self, field: &BitField) -> TokenStream2 {
        if !field.permissions.has_write() {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            uint_ty,
            additional_ty,
            size,
            offset,
            ..
        } = field;
        let offset =
            offset.expect("offset must be set before code generation");
        let fn_name = format_ident!("set_{}", name);
        let struct_type = &self.struct_type;
        let ty = additional_ty
            .as_ref()
            .map_or(uint_ty as &dyn ToTokens, |a| a);

        quote! {
            #vis fn #fn_name(&mut self, v: #ty) {
                debug_assert!(
                    (v as usize) < (1 << #size),
                    "Value is too large for this bitfield"
                );
                unsafe {
                    let addr = self as *const _ as *mut #struct_type;
                    let val = std::ptr::read_volatile(addr);
                    let cleared = val & !(((1 << #size) - 1) << #offset);
                    let new = cleared | ((v as #struct_type) << #offset);
                    std::ptr::write_volatile(addr, new);
                }
            }
        }
    }

    fn fn_clear(&self, field: &'a BitField) -> TokenStream2 {
        let Some(clear_val) = field.permissions.has_clear() else {
            return TokenStream2::new();
        };

        let BitField {
            vis,
            name,
            size,
            offset,
            ..
        } = field;
        let offset =
            offset.expect("offset must be set before code generation");
        let fn_name = format_ident!("clear_{}", name);
        let struct_type = &self.struct_type;

        quote! {
            #vis fn #fn_name(&mut self) {
                unsafe {
                    let addr = self as *const _ as *mut #struct_type;
                    let val = core::ptr::read_volatile(addr);
                    let cleared = val & !(((1 << #size) - 1) << #offset);
                    let new = cleared | ((#clear_val as #struct_type) << #offset);
                    std::ptr::write_volatile(addr, new);
                }
            }
        }
    }

    fn debug_impl(&self) -> TokenStream2 {
        let struct_name = self.struct_name;
        let field_names: Vec<_> =
            self.fields.iter().map(|f| f.name).collect();
        let getters: Vec<_> = field_names
            .iter()
            .map(|n| format_ident!("get_{}", n))
            .collect();

        quote! {
            impl std::fmt::Debug for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!(#struct_name))
                        #(.field(stringify!(#field_names), &self.#getters()))*
                        .finish()
                }
            }
        }
    }
}

// ─── Helpers
// ──────────────────────────────────────────────────────────────────

/// Given a type of the form `B<n>`, returns the smallest `u*` type that
/// fits `n` bits, along with `n` itself.
fn get_closest_uint(ty: &Type) -> syn::Result<(TypePath, usize)> {
    let Type::Path(ty_path) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "Expected a single-ident type (e.g. `B8`)",
        ));
    };

    let ident = ty_path.path.get_ident().ok_or_else(|| {
        syn::Error::new_spanned(
            ty_path,
            "Expected a single-ident type (e.g. `B8`)",
        )
    })?;

    let type_name = ident.to_string();
    let bit_str = type_name.strip_prefix('B').ok_or_else(|| {
        syn::Error::new_spanned(
            ident,
            "Type must start with `B` (e.g. `B8`)",
        )
    })?;

    let bits: usize = bit_str.parse().map_err(|_| {
        syn::Error::new_spanned(
            ident,
            "Cannot parse bit count from type name",
        )
    })?;

    let uint_ty = match bits {
        0..=8 => parse_quote!(u8),
        9..=16 => parse_quote!(u16),
        17..=32 => parse_quote!(u32),
        33..=64 => parse_quote!(u64),
        65..=128 => parse_quote!(u128),
        _ => {
            return Err(syn::Error::new_spanned(
                ty_path,
                "Bit width must be between 0 and 128",
            ));
        }
    };

    Ok((uint_ty, bits))
}

pub fn bitfields_impl(s: ItemStruct) -> syn::Result<TokenStream2> {
    let bitfield = Bitflags::try_from(&s)?;
    let min_uint = &bitfield.struct_type;
    let vis = &s.vis;
    let ident = &s.ident;

    let methods = bitfield.fields.iter().map(|b| {
        let read = bitfield.fn_read(b);
        let write = bitfield.fn_write(b);
        let clear = bitfield.fn_clear(b);
        quote! { #read #write #clear }
    });

    let debug_impl = bitfield.debug_impl();

    Ok(quote! {
        #vis struct #ident(#min_uint);

        impl #ident {
            pub fn new() -> Self {
                Self(0)
            }

            #(#methods)*
        }

        #debug_impl
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
                c: B1,
            }
        };

        let input = syn::parse2(example).unwrap();
        let output_tokens = bitfields_impl(input).unwrap();
        println!("{:#?}", output_tokens);
    }
}
