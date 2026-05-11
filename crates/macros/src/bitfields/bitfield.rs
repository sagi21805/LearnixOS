use syn::{Field, Ident, Meta, TypePath, Visibility};

use crate::bitfields::{
    flag_attr::FlagAttribute, utils::get_closest_uint,
};

pub struct BitField<'a> {
    pub attr: FlagAttribute,
    pub doc_attrs: Vec<syn::Attribute>,
    pub vis: &'a Visibility,
    pub name: &'a Ident,
    pub uint_ty: Box<TypePath>,
    pub size: usize,
    pub offset: Option<usize>,
}

impl<'a> TryFrom<&'a Field> for BitField<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a Field) -> Result<Self, Self::Error> {
        let (min_uint, size) = get_closest_uint(&value.ty)?;
        let name = value.ident.as_ref().expect("Field must have a name");

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
            return Ok(BitField {
                attr: FlagAttribute::default(),
                vis: &value.vis,
                name,
                uint_ty: Box::new(min_uint),
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

            Ok(BitField {
                attr: syn::parse2::<FlagAttribute>(list.tokens.clone())?,
                vis: &value.vis,
                name,
                uint_ty: Box::new(min_uint),
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
