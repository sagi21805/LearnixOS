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
    struct_type: Box<TypePath>,
    fields: Vec<BitField<'a>>,
}

impl<'a> TryFrom<&'a ItemStruct> for BitFields<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a ItemStruct) -> Result<Self, Self::Error> {
        let mut offset = 0;
        let fields = value
            .fields
            .iter()
            .map(|f| {
                let field = BitField::new(f, offset)?;
                offset += field.ty.size;
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
    fn checks(&self, field: &'a BitField) -> TokenStream2 {
        let offset = field.offset;
        let size = field.ty.size;
        let struct_type = &self.struct_type;

        let mut checks = quote! {
            debug_assert!(
                (
                   (v as #struct_type) >> #offset
                ) < (1 << #size) as #struct_type,
                "Value is too large for this bitfield"
            );
        };

        if field.attr.dont_shift {
            checks.extend(quote! {
                debug_assert!(
                    v & !((((1 << #size) - 1) as #struct_type) << #offset) == 0,
                    "Value overrides flags on positions that are not in bounds of flag",
                );
            });
        }

        checks
    }

    /// Returns the types needed to represent the item.
    /// (The actual type, uint repr type, struct type)
    fn types(
        &self,
        field: &'a BitField,
    ) -> (&TypePath, &TypePath, &TypePath) {
        let repr_ty = if field.attr.dont_shift {
            &*self.struct_type
        } else {
            field.ty.repr_ty.as_ref()
        };
        let ty = field.attr.flag_type.as_deref().unwrap_or(repr_ty);
        (ty, repr_ty, self.struct_type.as_ref())
    }

    fn shift(&self, field: &'a BitField) -> TokenStream2 {
        let offset = field.offset;

        // We literly don't shift
        if field.attr.dont_shift {
            quote! {}
        } else {
            quote! { << #offset }
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

        let (ty, repr_ty, struct_type) = self.types(field);
        let checks = self.checks(field);
        let shift = self.shift(field);

        quote! {
            #(#doc_attrs)*
            #[inline]
            #vis const fn #name(mut self, v: #ty) -> Self {
                let v = #repr_ty::try_from(v)
                    .ok()
                    .expect("Can't convery value 'v' into the struct type");

                #checks

                self.0 |= (v as #struct_type) #shift;
                self
            }
        }
    }

    fn fn_read(&self, field: &'a BitField) -> TokenStream2 {
        if !field.attr.permissions.read {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            ty,
            offset,
            doc_attrs,
            ..
        } = field;

        let size = ty.size;
        let (ty, repr_ty, struct_type) = self.types(field);
        let shift = self.shift(field);

        let fn_name = if ty.path.get_ident().is_some_and(|i| i == "bool") {
            format_ident!("is_{}", name)
        } else {
            format_ident!("get_{}", name)
        };

        quote! {
            #(#doc_attrs)*
            #[inline]
            #vis fn #fn_name(&self) -> #ty {
                unsafe {
                    let addr = self as *const _ as *mut #struct_type;
                    let val = core::ptr::read_volatile(addr);
                    #ty::try_from(
                        (
                            (
                            val & (((1 << #size) - 1) << #offset)
                            ) #shift
                        ) as #repr_ty
                    ).expect("Cannot convert bit representation into the given type")
                }
            }
        }
    }

    fn fn_write(&self, field: &BitField) -> TokenStream2 {
        if !field.attr.permissions.write {
            return TokenStream2::new();
        }

        let BitField {
            vis,
            name,
            ty,
            offset,
            doc_attrs,
            ..
        } = field;

        let fn_name = format_ident!("set_{}", name);
        let size = ty.size;
        let (ty, repr_ty, struct_type) = self.types(field);
        let shift = self.shift(field);
        let checks = self.checks(field);

        quote! {
            #(#doc_attrs)*
            #[inline]
            #vis fn #fn_name(&mut self, v: #ty) {
                let v = #repr_ty::try_from(v)
                    .ok()
                    .expect("Can't convery value 'v' into the struct type");

                #checks

                unsafe {
                    let addr = self as *const _ as *mut #struct_type;
                    let val = ::core::ptr::read_volatile(addr);
                    let cleared = val & !(((1 << #size) - 1) << #offset);
                    let new = cleared | ((#repr_ty::try_from(v).unwrap() as #struct_type) #shift);
                    ::core::ptr::write_volatile(addr, new);
                }
            }
        }
    }

    fn fn_clear(&self, field: &'a BitField) -> TokenStream2 {
        let Some(clear_val) = field.attr.permissions.clear else {
            return TokenStream2::new();
        };

        let BitField {
            vis,
            name,
            ty,
            offset,
            doc_attrs,
            ..
        } = field;
        let fn_name = format_ident!("clear_{}", name);
        let size = ty.size;
        let (_, repr_ty, struct_type) = self.types(field);
        let checks = self.checks(field);
        let shift = self.shift(field);

        quote! {
            #(#doc_attrs)*
            #[inline]
            #vis fn #fn_name(&mut self) {
                let v = #repr_ty::try_from(#clear_val)
                    .ok()
                    .expect("Can't convery value 'v' into the struct type");

                #checks

                unsafe {
                    let addr = self as *const _ as *mut #struct_type;
                    let val = ::core::ptr::read_volatile(addr);
                    let cleared = val & !(((1 << #size) - 1) << #offset);
                    let new = cleared | ((#clear_val as #struct_type) #shift);
                    ::core::ptr::write_volatile(addr, new);
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
                let getter = if f.ty.size == 1
                        && f.attr.flag_type
                            .as_ref()
                            .is_some_and(|t| t.path
                                .get_ident()
                                .is_some_and(|i| i == "bool"))
                {
                    format_ident!("is_{}", f.name)
                } else {
                    format_ident!("get_{}", f.name)
                };
                let name = f.name;
                let ty = f.attr.flag_type.as_ref().unwrap_or(&f.ty.repr_ty);
                quote! { stringify!(#name), &#ty::try_from(self.#getter()) }
            })
            .collect();

        quote! {
            impl ::core::fmt::Debug for #struct_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(stringify!(#struct_name))
                        #(.field(#field_args))*
                        .finish()
                }
            }
        }
    }

    fn from_impl(&self) -> TokenStream2 {
        let BitFields {
            struct_name,
            struct_type,
            ..
        } = self;
        quote! {
            impl const ::core::convert::From<#struct_type> for #struct_name {
                fn from(value: #struct_type) -> Self {
                    #struct_name(value)
                }
            }
        }
    }

    // impls from for the other type instead of into.
    fn into_impl(&self) -> TokenStream2 {
        let BitFields {
            struct_name,
            struct_type,
            ..
        } = self;
        quote! {
            impl const ::core::convert::From<#struct_name> for #struct_type {
                fn from(value: #struct_name) -> #struct_type {
                    value.0
                }
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

        let methods = self.fields.iter().map(|f| {
            let read = self.fn_read(f);
            let write = self.fn_write(f);
            let clear = self.fn_clear(f);
            let build = self.fn_build(f);
            quote! { #read #write #clear #build }
        });

        let debug_impl = self.debug_impl();
        let from_impl = self.from_impl();
        let into_impl = self.into_impl();

        tokens.extend(quote! {
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
        });
    }
}
