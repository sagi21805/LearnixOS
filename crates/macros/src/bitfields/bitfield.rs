use syn::{Field, Ident, Visibility, parse_quote, spanned::Spanned};

use crate::bitfields::{flag_attr::FlagAttribute, utils::FlagMeta};

pub struct BitField<'a> {
    pub attr: FlagAttribute,
    pub doc_attrs: Vec<&'a syn::Attribute>,
    pub vis: &'a Visibility,
    pub name: &'a Ident,
    pub meta: FlagMeta,
    pub offset: usize,
}

fn extract_attributes(
    f: &Field,
) -> syn::Result<(Option<&syn::Attribute>, Vec<&syn::Attribute>)> {
    let doc_attrs: Vec<&syn::Attribute> = f
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .collect();

    let flag_attrs: Vec<&syn::Attribute> = f
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("doc"))
        .collect();

    if flag_attrs.len() > 1 {
        return Err(syn::Error::new_spanned(
            flag_attrs[1],
            "Fields must have at most one attribute",
        ));
    }

    Ok((flag_attrs.get(0).copied(), doc_attrs))
}

impl<'a> BitField<'a> {
    pub fn new(f: &'a Field, offset: usize) -> syn::Result<Self> {
        let name = f.ident.as_ref().ok_or(syn::Error::new(
            f.span(),
            "Struct field must have a name",
        ))?;
        let meta: FlagMeta = (&f.ty).try_into()?;

        let (flag_attrs, doc_attrs) = extract_attributes(f)?;

        let mut attr = if let Some(flag_attr) = flag_attrs {
            FlagAttribute::try_from(&flag_attr.meta)?
        } else {
            FlagAttribute::default()
        };

        // If the flag type is not specified and the width is 1, we add
        // `bool` as the flag type for convenience.
        if attr.flag_type.is_none() && meta.width == 1 {
            attr.flag_type = Some(parse_quote!(bool))
        }

        Ok(BitField {
            attr,
            vis: &f.vis,
            name,
            meta,
            offset,
            doc_attrs,
        })
    }
}
