use syn::{Type, TypePath, parse_quote};

pub struct BitSize {
    /// The type that represents the bit size.
    /// For example, `B2` is represented by `u8`, and `B9` is represented
    /// by `u16`
    pub repr_ty: Box<TypePath>,
    /// The actual size of the bit field, in bits.
    pub size: usize,
}

impl<'a> TryFrom<&'a Type> for BitSize {
    type Error = syn::Error;

    fn try_from(ty: &'a Type) -> syn::Result<Self> {
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

        let size: usize = bit_str.parse().map_err(|_| {
            syn::Error::new_spanned(
                ident,
                "Cannot parse bit count from type name",
            )
        })?;

        let repr_ty = match size {
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

        Ok(BitSize { repr_ty, size })
    }
}
