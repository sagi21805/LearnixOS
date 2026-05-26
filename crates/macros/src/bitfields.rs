mod bitfield;
mod flag_attr;
mod utils;

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, Ident, ItemStruct, TypePath, Visibility};

use crate::bitfields::bitfield::BitField;

pub struct BitFields<'a> {
    attrs: &'a Vec<Attribute>,
    vis: &'a Visibility,
    struct_name: &'a Ident,
    struct_type: TypePath,
    fields: Vec<BitField<'a>>,
}

/// The three types needed to generate code for a given field.
struct FieldTypes<'a> {
    /// The user-facing type (e.g. a flag enum or `bool`).
    ty: &'a TypePath,
    /// The primitive unsigned integer used as the wire representation.
    repr_ty: &'a TypePath,
    /// The struct's backing unsigned integer type.
    struct_ty: &'a TypePath,
}

impl<'a> TryFrom<&'a ItemStruct> for BitFields<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a ItemStruct) -> syn::Result<Self> {
        let mut offset = 0;
        let fields = value
            .fields
            .iter()
            .map(|f| {
                let field = BitField::new(f, offset)?;
                offset += field.meta.width;
                Ok(field)
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(BitFields {
            attrs: &value.attrs,
            vis: &value.vis,
            struct_name: &value.ident,
            struct_type: utils::type_from_size(offset)?,
            fields,
        })
    }
}

impl<'a> BitFields<'a> {
    fn field_types<'b>(&'b self, field: &'a BitField) -> FieldTypes<'b> {
        let repr_ty = if field.attr.dont_shift {
            &self.struct_type
        } else {
            &field.meta.repr_ty
        };
        FieldTypes {
            ty: field.attr.flag_type.as_ref().unwrap_or(repr_ty),
            repr_ty,
            struct_ty: &self.struct_type,
        }
    }

    fn checks(&self, field: &'a BitField, fn_name: &str) -> TokenStream2 {
        let struct_name = self.struct_name;
        let FieldTypes { struct_ty, .. } = self.field_types(field);
        let width = field.meta.width;
        let offset = field.offset;
        let max_val = u128::MAX >> (u128::BITS - width as u32);

        let except_msg = format!(
            "{struct_name}::{fn_name}: value out of range: must fit in \
             {width} bits (max {max_val:#x})"
        );

        let mut checks = quote! {
            debug_assert!(
                (v as #struct_ty) <= (#max_val as #struct_ty),
                #except_msg,
            );
        };

        if field.attr.dont_shift {
            let field_mask = max_val << offset;
            let except_msg = format!(
                "{struct_name}::{fn_name}: value contains bits outside \
                 the {width}-bit field at bit offset {offset} (permitted \
                 mask: {field_mask:#x})"
            );
            checks.extend(quote! {
                debug_assert!(
                    v & !(((#max_val) as #struct_ty) << #offset) == 0,
                    #except_msg,
                );
            });
        }

        checks
    }

    /// Shift expression for writing a value into the backing integer.
    fn write_shift(&self, field: &'a BitField) -> TokenStream2 {
        if field.attr.dont_shift {
            quote! {}
        } else {
            let offset = field.offset;
            quote! { << #offset }
        }
    }

    /// Shift expression for reading a value out of the backing integer.
    fn read_shift(&self, field: &'a BitField) -> TokenStream2 {
        if field.attr.dont_shift {
            quote! {}
        } else {
            let offset = field.offset;
            quote! { >> #offset }
        }
    }

    fn v_to_repr(&self, field: &'a BitField) -> TokenStream2 {
        let FieldTypes { ty, repr_ty, .. } = self.field_types(field);
        let expect_msg = format!(
            "Can't convert value 'v' ({}) into {}",
            quote!(#ty),
            quote!(#repr_ty),
        );
        quote! {
            let v = <#repr_ty as ::core::convert::TryFrom<_>>::try_from(v)
                .ok()
                .expect(#expect_msg);
        }
    }

    fn volatile_write(
        &self,
        field: &'a BitField,
        v: TokenStream2,
    ) -> TokenStream2 {
        let offset = field.offset;
        let width = field.meta.width;
        let struct_ty = &self.struct_type;
        let shift = self.write_shift(field);

        quote! {
            unsafe {
                let addr = self as *const _ as *mut #struct_ty;
                let val = ::core::ptr::read_volatile(addr);
                let cleared = val & !((#struct_ty::MAX >> (#struct_ty::BITS - #width as u32)) << #offset);
                let new = cleared | ((#v as #struct_ty) #shift);
                ::core::ptr::write_volatile(addr, new);
            }
        }
    }

    /// Returns the read accessor name (`is_<name>` for bool fields,
    /// `get_<name>` otherwise).
    fn read_fn_name(&self, field: &'a BitField) -> Ident {
        let FieldTypes { ty, .. } = self.field_types(field);
        if ty.path.get_ident().is_some_and(|i| i == "bool") {
            format_ident!("is_{}", field.name)
        } else {
            format_ident!("get_{}", field.name)
        }
    }

    fn fn_build(&self, field: &'a BitField) -> TokenStream2 {
        if !field.attr.permissions.write {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            doc_attrs,
            ..
        } = field;
        let FieldTypes { ty, struct_ty, .. } = self.field_types(field);
        let checks = self.checks(field, &name.to_string());
        let v_to_repr = self.v_to_repr(field);
        let shift = self.write_shift(field);

        quote! {
            #(#doc_attrs)*
            #[inline]
            #[track_caller]
            #vis const fn #name(mut self, v: #ty) -> Self {
                #checks
                #v_to_repr
                self.0 |= (v as #struct_ty) #shift;
                self
            }
        }
    }

    fn fn_read(&self, field: &'a BitField) -> TokenStream2 {
        if !field.attr.permissions.read {
            return TokenStream2::new();
        }

        let BitField { vis, doc_attrs, .. } = field;
        let FieldTypes {
            ty,
            repr_ty,
            struct_ty,
        } = self.field_types(field);
        let fn_name = self.read_fn_name(field);
        let width = field.meta.width;
        let offset = field.offset;
        let shift = self.read_shift(field);
        let expect_msg = format!(
            "Cannot convert bit representation into {}",
            quote!(#ty)
        );

        quote! {
            #(#doc_attrs)*
            #[inline]
            #[track_caller]
            #vis fn #fn_name(&self) -> #ty {
                unsafe {
                    let addr = self as *const _ as *mut #struct_ty;
                    let val = ::core::ptr::read_volatile(addr);
                    let bits = (val & ((#struct_ty::MAX >> (#struct_ty::BITS - #width as u32)) << #offset)) #shift;
                    <#ty as ::core::convert::TryFrom<#repr_ty>>::try_from(bits as #repr_ty)
                        .expect(#expect_msg)
                }
            }
        }
    }

    fn fn_write(&self, field: &'a BitField) -> TokenStream2 {
        if !field.attr.permissions.write {
            return TokenStream2::new();
        }

        let BitField { vis, doc_attrs, .. } = field;
        let fn_name = format_ident!("set_{}", field.name);
        let FieldTypes { ty, .. } = self.field_types(field);
        let checks = self.checks(field, &fn_name.to_string());
        let v_to_repr = self.v_to_repr(field);
        let write = self.volatile_write(field, quote! { v });

        quote! {
            #(#doc_attrs)*
            #[inline]
            #[track_caller]
            #vis fn #fn_name(&mut self, v: #ty) {
                #checks
                #v_to_repr
                #write
            }
        }
    }

    fn fn_clear(&self, field: &'a BitField) -> TokenStream2 {
        let Some(clear_val) = field.attr.permissions.clear else {
            return TokenStream2::new();
        };

        let BitField { vis, doc_attrs, .. } = field;
        let fn_name = format_ident!("clear_{}", field.name);
        let FieldTypes { repr_ty, .. } = self.field_types(field);
        let checks = self.checks(field, &fn_name.to_string());
        let write = self.volatile_write(field, quote! { #clear_val });

        quote! {
            #(#doc_attrs)*
            #[inline]
            #[track_caller]
            #vis fn #fn_name(&mut self) {
                let v = #clear_val as #repr_ty;
                #checks
                #write
            }
        }
    }

    fn debug_impl(&self) -> TokenStream2 {
        let struct_name = self.struct_name;
        let fields = self.fields.iter().map(|f| {
            let getter = self.read_fn_name(f);
            let name = f.name;
            quote! { .field(stringify!(#name), &self.#getter()) }
        });

        quote! {
            impl ::core::fmt::Debug for #struct_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_struct(stringify!(#struct_name))
                        #(#fields)*
                        .finish()
                }
            }
        }
    }

    fn conversion_impls(&self) -> TokenStream2 {
        let struct_name = self.struct_name;
        let struct_type = &self.struct_type;
        quote! {
            impl const ::core::convert::From<#struct_type> for #struct_name {
                fn from(value: #struct_type) -> Self { #struct_name(value) }
            }
            impl const ::core::convert::From<#struct_name> for #struct_type {
                fn from(value: #struct_name) -> Self { value.0 }
            }
        }
    }
}

impl<'a> ToTokens for BitFields<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let BitFields {
            attrs,
            vis,
            struct_name,
            struct_type,
            ..
        } = self;

        let methods = self.fields.iter().map(|field| {
            let read = self.fn_read(field);
            let write = self.fn_write(field);
            let clear = self.fn_clear(field);
            let build = self.fn_build(field);
            quote! { #read #write #clear #build }
        });

        let debug_impl = self.debug_impl();
        let conversion_impls = self.conversion_impls();

        tokens.extend(quote! {
            #(#attrs)*
            #[repr(transparent)]
            #[derive(Copy, Clone)]
            #vis struct #struct_name(#struct_type);

            impl #struct_name {
                #[inline]
                pub fn new() -> Self { Self(0) }

                #(#methods)*
            }

            impl const Default for #struct_name {
                fn default() -> Self { Self(0) }
            }

            #conversion_impls

            #debug_impl
        });
    }
}
