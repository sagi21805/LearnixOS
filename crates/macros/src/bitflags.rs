use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Attribute, Field, Ident, ItemStruct, LitInt, Meta, Path, Token, Type,
    TypePath, Visibility,
    parse::{Parse, discouraged::Speculative},
    parse_quote,
};

mod keyword {
    syn::custom_keyword!(flag);
    syn::custom_keyword!(flag_type);
    syn::custom_keyword!(dont_shift);
}

#[derive(Debug)]
struct FlagPermission {
    read: bool,
    write: bool,
    clear: Option<usize>,
}

impl Default for FlagPermission {
    fn default() -> FlagPermission {
        FlagPermission {
            read: true,
            write: true,
            clear: None,
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
        let mut flag_permissions = FlagPermission::default();

        let permission =
            input.parse::<Ident>()?.to_string().to_lowercase();

        if !permission.chars().all(|c| matches!(c, 'r' | 'w' | 'c')) {
            return Err(syn::Error::new_spanned(
                &permission,
                "expected permission string (e.g. `rw`, `r`, `wc(0)`)",
            ));
        }

        match (permission.contains('r'), permission.contains('w')) {
            (true, false) => flag_permissions.write = false,
            (false, true) => flag_permissions.read = false,
            (false, false) => {
                flag_permissions.read = false;
                flag_permissions.write = false;
            }
            (true, true) => { /*default*/ }
        };

        if permission.contains('c') {
            let content;
            let _ = syn::parenthesized!(content in input);
            let int =
                content.parse::<LitInt>()?.base10_parse::<usize>()?;
            flag_permissions.clear = Some(int);
        }

        Ok(flag_permissions)
    }
}

#[derive(Debug)]
pub struct FlagType {
    _flag_type_token: keyword::flag_type,
    _equal: Token![=],
    ty: Box<TypePath>,
}

impl Parse for FlagType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(FlagType {
            _flag_type_token: input.parse()?,
            _equal: input.parse()?,
            ty: input.parse()?,
        })
    }
}

#[derive(Default, Debug)]
pub struct FlagAttribute {
    permissions: FlagPermission,
    flag_type: Option<Box<TypePath>>,
    dont_shift: bool,
}

impl Parse for FlagAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attributes = FlagAttribute::default();

        // Track what's already been parsed
        let mut seen_permissions: Option<proc_macro2::Span> = None;
        let mut seen_flag_type: Option<proc_macro2::Span> = None;
        let mut seen_dont_shift: Option<proc_macro2::Span> = None;

        while !input.is_empty() {
            let mut error_count = 0;
            'next: {
                let fork = input.fork();
                match fork.parse::<FlagPermission>() {
                    Ok(permissions) => {
                        if let Some(first_span) = seen_permissions {
                            return Err(syn::Error::new(
                                first_span,
                                "duplicate `permissions` option",
                            ));
                        }
                        seen_permissions = Some(input.span());
                        attributes.permissions = permissions;
                        input.advance_to(&fork);
                        break 'next;
                    }
                    Err(_) => error_count += 1,
                }

                let fork = input.fork();
                match fork.parse::<FlagType>() {
                    Ok(flag_type) => {
                        if let Some(first_span) = seen_flag_type {
                            return Err(syn::Error::new(
                                first_span,
                                "duplicate `flag_type` option",
                            ));
                        }
                        seen_flag_type = Some(input.span());
                        attributes.flag_type = Some(flag_type.ty);
                        input.advance_to(&fork);
                        break 'next;
                    }
                    Err(_) => error_count += 1,
                }

                let fork = input.fork();
                match fork.parse::<keyword::dont_shift>() {
                    Ok(kw) => {
                        if let Some(first_span) = seen_dont_shift {
                            return Err(syn::Error::new(
                                first_span,
                                "duplicate `dont_shift` option",
                            ));
                        }
                        seen_dont_shift = Some(input.span());
                        attributes.dont_shift = true;
                        input.advance_to(&fork);
                        break 'next;
                    }
                    Err(_) => error_count += 1,
                }
            }

            if error_count == 3 {
                let unknown: proc_macro2::TokenTree = input.parse()?;
                return Err(syn::Error::new_spanned(
                    &unknown,
                    format!("unknown option: {}", unknown),
                ));
            }

            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(attributes)
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
    dont_shift: bool,
    size: usize,
    offset: Option<usize>,
    doc_attrs: Vec<syn::Attribute>,
}

impl<'a> TryFrom<&'a Field> for BitField<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a Field) -> Result<Self, Self::Error> {
        let (min_uint, size) = get_closest_uint(&value.ty)?;
        let name = value.ident.as_ref().expect("Fields must have a name");

        let doc_attrs: Vec<syn::Attribute> = value
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .cloned()
            .collect();

        let flag_attrs: Vec<&syn::Attribute> = value
            .attrs
            .iter()
            .filter(|a| !a.path().is_ident("doc"))
            .collect();

        if value.attrs.is_empty() {
            return Ok(BitField {
                permissions: FlagPermission::default(),
                vis: &value.vis,
                name,
                dont_shift: false,
                uint_ty: Box::new(min_uint),
                additional_ty: None,
                size,
                offset: None,
                doc_attrs,
            });
        }

        if flag_attrs.len() > 1 {
            return Err(syn::Error::new_spanned(
                value,
                "Fields must have at most one attribute",
            ));
        }

        let attr = flag_attrs[0];
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
                dont_shift,
            } = syn::parse2::<FlagAttribute>(list.tokens.clone())?;

            Ok(BitField {
                permissions,
                vis: &value.vis,
                name,
                uint_ty: Box::new(min_uint),
                additional_ty: flag_type,
                dont_shift,
                size,
                offset: None,
                doc_attrs,
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
    attrs: &'a Vec<Attribute>,
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
            attrs: &value.attrs,
            struct_name: &value.ident,
            struct_type: Box::new(struct_type),
            fields,
        })
    }
}

impl<'a> Bitflags<'a> {
    fn fn_build(&self, field: &'a BitField) -> TokenStream2 {
        if !field.permissions.write {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            uint_ty,
            additional_ty,
            dont_shift,
            size,
            offset,
            doc_attrs,
            ..
        } = field;
        let offset =
            offset.expect("offset must be set before code generation");
        let struct_type = &self.struct_type;
        let ty = additional_ty.as_ref().unwrap_or(uint_ty);
        if *size == 1 && additional_ty.is_none() {
            quote! {
                #[inline]
                #vis const fn #name(mut self) -> Self {
                    self.0 |= (1 << #offset);
                    self
                }
            }
        } else if *dont_shift {
            quote! {
                #(#doc_attrs)*
                #[inline]
                #vis const fn #name(mut self, v: #struct_type) -> Self {
                    debug_assert!(
                        (#uint_ty::try_from(v).ok().expect("Can't convery value 'v' into the struct type") as #struct_type >> #offset) < (1 << #size) as #struct_type,
                        "Value is too large for this bitfield"
                    );
                    debug_assert!(
                        (v & !((((1 << #size) - 1) as #struct_type) << #offset)) == 0,
                        "Value overrides flags on positions that are not in bounds of flag",
                    );
                    self.0 |= #uint_ty::try_from(v).ok().expect("Can't convery value 'v' into the struct type") as #struct_type;
                    self
                }
            }
        } else {
            quote! {
                #(#doc_attrs)*
                #[inline]
                #vis const fn #name(mut self, v: #ty) -> Self {
                    debug_assert!(
                        (#uint_ty::try_from(v).ok().expect("Can't convery value 'v' into the struct type") as #struct_type)  < (1 << #size) as #struct_type,
                        "Value is too large for this bitfield"
                    );
                    self.0 |= ((#uint_ty::try_from(v).ok().expect("Can't convery value 'v' into the struct type") as #struct_type) << #offset);
                    self
                }
            }
        }
    }

    fn fn_read(&self, field: &'a BitField) -> TokenStream2 {
        if !field.permissions.read {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            uint_ty,
            size,
            offset,
            dont_shift,
            additional_ty,
            doc_attrs,
            ..
        } = field;
        let offset =
            offset.expect("offset must be set before code generation");
        let struct_type = &self.struct_type;
        if *size == 1 && additional_ty.is_none() {
            let fn_name = format_ident!("is_{}", name);
            quote! {
                #(#doc_attrs)*
                #[inline]
                #vis fn #fn_name(&self) -> bool {
                    unsafe {
                        let addr = self as *const _ as *mut #struct_type;
                        let val = core::ptr::read_volatile(addr);
                        val & (1 << #offset) != 0
                    }
                }
            }
        } else {
            let fn_name = format_ident!("get_{}", name);
            if *dont_shift {
                let ty = additional_ty.as_ref().unwrap_or(struct_type);
                quote! {
                    #(#doc_attrs)*
                    #[inline]
                    #vis fn #fn_name(&self) -> #ty {
                        unsafe {
                            let addr = self as *const _ as *mut #struct_type;
                            let val = core::ptr::read_volatile(addr);
                            #ty::try_from((val & (((1 << #size) - 1) << #offset))).expect("Cannot convert bit representation into the given type")
                        }
                    }
                }
            } else {
                let ty = additional_ty.as_ref().unwrap_or(uint_ty);
                quote! {
                    #(#doc_attrs)*
                    #[inline]
                    #vis fn #fn_name(&self) -> #ty {
                        unsafe {
                            let addr = self as *const _ as *mut #struct_type;
                            let val = core::ptr::read_volatile(addr);
                            #ty::try_from(((val >> #offset) & ((1 << #size) - 1)) as #uint_ty).expect("Cannot convert bit representation into the given type")
                        }
                    }
                }
            }
        }
    }

    fn fn_write(&self, field: &BitField) -> TokenStream2 {
        if !field.permissions.write {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            uint_ty,
            additional_ty,
            size,
            dont_shift,
            offset,
            doc_attrs,
            ..
        } = field;
        let offset =
            offset.expect("offset must be set before code generation");
        let fn_name = format_ident!("set_{}", name);
        let struct_type = &self.struct_type;
        let mut ty = additional_ty.as_ref().unwrap_or(uint_ty);

        if *dont_shift && *size != 1 {
            quote! {
                #(#doc_attrs)*
                #[inline]
                #vis fn #fn_name(&mut self, v: #struct_type) {
                    debug_assert!(
                        (#uint_ty::try_from(v).ok().expect("Can't convery value 'v' into the struct type") as #struct_type >> #offset) < (1 << #size) as #struct_type,
                        "Value: {:?} is too large for this bitfield",
                        v >> #offset
                    );
                    debug_assert!(
                        (v & !((((1 << #size) - 1) as #struct_type) << #offset)) == 0,
                        "Value: {:?} overrides flags on positions that are not in bounds of flag {}",
                        v, stringify!(#name)
                    );
                    unsafe {
                        let addr = self as *const _ as *mut #struct_type;
                        let val = core::ptr::read_volatile(addr);
                        let cleared = val & !(((1 << #size) - 1) << #offset);
                        let new = cleared | (#uint_ty::try_from(v).unwrap() as #struct_type);
                        core::ptr::write_volatile(addr, new);
                    }
                }
            }
        } else {
            let bool_type: Box<TypePath> = Box::new(parse_quote!(bool));
            if *size == 1 {
                ty = &bool_type
            }
            quote! {
                #(#doc_attrs)*
                #[inline]
                #vis fn #fn_name(&mut self, v: #ty) {
                    debug_assert!(
                        (#uint_ty::try_from(v).ok().expect("Can't convery value 'v' into the struct type") as #struct_type) < (1 << #size) as #struct_type,
                        "Value: {:?} is too large for this bitfield",
                        v
                    );
                    unsafe {
                        let addr = self as *const _ as *mut #struct_type;
                        let val = core::ptr::read_volatile(addr);
                        let cleared = val & !(((1 << #size) - 1) << #offset);
                        let new = cleared | ((#uint_ty::try_from(v).unwrap() as #struct_type) << #offset);
                        core::ptr::write_volatile(addr, new);
                    }
                }
            }
        }
    }

    fn fn_clear(&self, field: &'a BitField) -> TokenStream2 {
        let Some(clear_val) = field.permissions.clear else {
            return TokenStream2::new();
        };

        let BitField {
            vis,
            name,
            size,
            offset,
            doc_attrs,
            ..
        } = field;
        let offset =
            offset.expect("offset must be set before code generation");
        let fn_name = format_ident!("clear_{}", name);
        let struct_type = &self.struct_type;

        quote! {
            #(#doc_attrs)*
            #[inline]
            #vis fn #fn_name(&mut self) {
                unsafe {
                    let addr = self as *const _ as *mut #struct_type;
                    let val = core::ptr::read_volatile(addr);
                    let cleared = val & !(((1 << #size) - 1) << #offset);
                    let new = cleared | ((#clear_val as #struct_type) << #offset);
                    core::ptr::write_volatile(addr, new);
                }
            }
        }
    }

    fn debug_impl(&self) -> TokenStream2 {
        let struct_name = self.struct_name;
        let field_args: Vec<_> = self
            .fields
            .iter()
            .map(|f| {
                let getter = if f.size == 1 && f.additional_ty.is_none() {
                    format_ident!("is_{}", f.name)
                } else {
                    format_ident!("get_{}", f.name)
                };
                let name = f.name;
                let ty = f.additional_ty.as_ref().unwrap_or(&f.uint_ty);
                quote! { stringify!(#name), &#ty::try_from(self.#getter()) }
            })
            .collect();

        quote! {
            impl core::fmt::Debug for #struct_name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(stringify!(#struct_name))
                        #(.field(#field_args))*
                        .finish()
                }
            }
        }
    }

    fn from_impl(&self) -> TokenStream2 {
        let Bitflags {
            struct_name,
            struct_type,
            ..
        } = self;
        quote! {
            impl const From<#struct_type> for #struct_name {
                fn from(value: #struct_type) -> Self {
                    #struct_name(value)
                }
            }
        }
    }

    // impls from for the other type instead of into.
    fn into_impl(&self) -> TokenStream2 {
        let Bitflags {
            struct_name,
            struct_type,
            ..
        } = self;
        quote! {
            impl const From<#struct_name> for #struct_type {
                fn from(value: #struct_name) -> #struct_type {
                    value.0
                }
            }
        }
    }
}

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
    let bitfield @ Bitflags {
        attrs,
        struct_name,
        struct_type,
        ..
    } = &Bitflags::try_from(&s)?;

    let vis = &s.vis;

    let methods = bitfield.fields.iter().map(|b| {
        let read = bitfield.fn_read(b);
        let write = bitfield.fn_write(b);
        let clear = bitfield.fn_clear(b);
        let build = bitfield.fn_build(b);
        quote! { #read #write #clear #build }
    });

    let debug_impl = bitfield.debug_impl();
    let from_impl = bitfield.from_impl();
    let into_impl = bitfield.into_impl();

    Ok(quote! {

        #(#attrs)*
        #[repr(transparent)]
        #[derive(Copy, Clone)]
        #vis struct #struct_name(#struct_type);

        impl #struct_name {
            #[inline]
            pub fn new() -> Self {
                Self(0)
            }

            #(#methods)*
        }

        impl const Default for #struct_name {
            fn default() -> Self {
                Self(0)
            }
        }

        #from_impl

        #into_impl

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
