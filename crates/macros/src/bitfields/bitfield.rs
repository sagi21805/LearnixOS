use syn::{Field, Ident, Visibility, parse_quote};

use crate::bitfields::{flag_attr::FlagAttribute, utils::BitSize};

pub struct BitField<'a> {
    pub attr: FlagAttribute,
    pub doc_attrs: Vec<&'a syn::Attribute>,
    pub vis: &'a Visibility,
    pub name: &'a Ident,
    pub ty: BitSize,
    pub offset: usize,
}

fn extract_attributes(
    f: &Field,
) -> (Vec<&syn::Attribute>, Vec<&syn::Attribute>) {
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

    (flag_attrs, doc_attrs)
}

impl<'a> BitField<'a> {
    pub fn new(f: &'a Field, offset: usize) -> syn::Result<Self> {
        let name = f.ident.as_ref().expect("Field must have a name");
        let ty: BitSize = (&f.ty).try_into()?;

        let (flag_attrs, doc_attrs) = extract_attributes(f);

        if flag_attrs.is_empty() {
            let mut attr = FlagAttribute::default();

            if attr.flag_type.is_none() && ty.size == 1 {
                attr.flag_type = Some(parse_quote!(bool))
            }

            return Ok(BitField {
                attr,
                vis: &f.vis,
                name,
                ty,
                offset,
                doc_attrs,
            });
        }

        if flag_attrs.len() > 1 {
            return Err(syn::Error::new_spanned(
                f,
                "Fields must have at most one attribute",
            ));
        }

        let attr = FlagAttribute::from_meta(&flag_attrs[0].meta, ty.size)?;

        Ok(BitField {
            attr,
            vis: &f.vis,
            name,
            ty,
            offset,
            doc_attrs,
        })
    }
}
