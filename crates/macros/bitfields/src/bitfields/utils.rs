use syn::{Type, TypePath, parse_quote};

pub struct FlagMeta {
    /// The type that represents the bit size.
    /// For example, `B2` is represented by `u8`, and `B9` is represented
    /// by `u16`
    pub repr_ty: TypePath,
    /// The actual size of the bit field, in bits.
    pub width: usize,
}

impl<'a> TryFrom<&'a Type> for FlagMeta {
    type Error = syn::Error;

    fn try_from(ty: &'a Type) -> syn::Result<Self> {
        let ident = match ty {
            Type::Path(syn::TypePath { path, .. }) => path.get_ident(),
            _ => None,
        }
        .ok_or_else(|| {
            syn::Error::new_spanned(
                ty,
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

        let size: usize = bit_str.parse().map_err(|_| {
            syn::Error::new_spanned(
                ident,
                "Cannot parse bit count from type name",
            )
        })?;

        let repr_ty = type_from_size(size)?;

        Ok(FlagMeta {
            repr_ty,
            width: size,
        })
    }
}

pub fn type_from_size(size: usize) -> syn::Result<TypePath> {
    match size {
        1..=8 => Ok(parse_quote!(u8)),
        9..=16 => Ok(parse_quote!(u16)),
        17..=32 => Ok(parse_quote!(u32)),
        33..=64 => Ok(parse_quote!(u64)),
        65..=128 => Ok(parse_quote!(u128)),
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Bit width must be between 1 and 128",
            ));
        }
    }
}
