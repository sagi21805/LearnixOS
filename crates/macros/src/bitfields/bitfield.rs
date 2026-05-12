use syn::{Field, Ident, Meta, Visibility, parse_quote};

use crate::bitfields::{flag_attr::FlagAttribute, utils::BitSize};

pub struct BitField<'a> {
    pub attr: FlagAttribute,
    pub doc_attrs: Vec<syn::Attribute>,
    pub vis: &'a Visibility,
    pub name: &'a Ident,
    pub ty: BitSize,
    pub offset: usize,
}

impl<'a> BitField<'a> {
    pub fn new(value: &'a Field, offset: usize) -> syn::Result<Self> {
        let name = value.ident.as_ref().expect("Field must have a name");
        let ty: BitSize = (&value.ty).try_into()?;

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

        if flag_attrs.is_empty() {
            let mut attr = FlagAttribute::default();

            if attr.flag_type.is_none() && ty.size == 1 {
                attr.flag_type = Some(parse_quote!(bool))
            }

            return Ok(BitField {
                attr,
                vis: &value.vis,
                name,
                ty,
                offset,
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

            let mut attr =
                syn::parse2::<FlagAttribute>(list.tokens.clone())?;
            if attr.flag_type.is_none() && ty.size == 1 {
                attr.flag_type = Some(parse_quote!(bool))
            }

            Ok(BitField {
                attr,
                vis: &value.vis,
                name,
                ty,
                offset,
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
