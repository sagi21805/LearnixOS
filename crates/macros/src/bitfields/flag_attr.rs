use syn::{
    Ident, LitInt, Meta, Token, TypePath,
    parse::{Parse, discouraged::Speculative},
    parse_quote,
};

mod keyword {
    syn::custom_keyword!(flag);
    syn::custom_keyword!(flag_type);
    syn::custom_keyword!(dont_shift);
}

#[derive(Default, Debug)]
pub struct FlagAttribute {
    pub permissions: FlagPermission,
    pub flag_type: Option<Box<TypePath>>,
    // TODO: Change into Option<keyword::dont_shift>
    pub dont_shift: bool,
}

fn try_parse<T: Parse>(
    input: syn::parse::ParseStream,
    seen: &mut Option<proc_macro2::Span>,
    error_count: &mut usize,
) -> Option<syn::Result<T>> {
    let fork = input.fork();
    let parsed = match fork.parse::<T>() {
        Ok(parsed) => parsed,
        Err(_) => {
            *error_count += 1;
            return None;
        }
    };

    if seen.is_some() {
        Some(Err(syn::Error::new(seen.unwrap(), "Duplicate attriubte")))
    } else {
        *seen = Some(input.span());
        input.advance_to(&fork);
        Some(Ok(parsed))
    }
}

impl Parse for FlagAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attributes = FlagAttribute::default();

        let mut seen_permissions: Option<proc_macro2::Span> = None;
        let mut seen_flag_type: Option<proc_macro2::Span> = None;
        let mut seen_dont_shift: Option<proc_macro2::Span> = None;

        while !input.is_empty() {
            let mut error_count = 0;
            let fp = try_parse::<FlagPermission>(
                input,
                &mut seen_permissions,
                &mut error_count,
            )
            .transpose()?;

            if let Some(permissions) = fp {
                attributes.permissions = permissions;
            }

            attributes.flag_type = try_parse::<FlagType>(
                input,
                &mut seen_flag_type,
                &mut error_count,
            )
            .transpose()?
            .map(|v| v.ty);

            attributes.dont_shift = try_parse::<keyword::dont_shift>(
                input,
                &mut seen_dont_shift,
                &mut error_count,
            )
            .transpose()?
            .is_some();

            // Couldn't parse any part of the attribute.
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

impl FlagAttribute {
    pub fn from_meta(
        meta: &Meta,
        size: usize,
    ) -> syn::Result<FlagAttribute> {
        if let Meta::List(list) = &meta {
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

            if attr.flag_type.is_none() && size == 1 {
                attr.flag_type = Some(parse_quote!(bool))
            }

            return Ok(attr);
        } else {
            Err(syn::Error::new_spanned(
                &meta,
                "Attribute must be in the form `flag(permission)` or \
                 `flag(permission, flag_type = Type)`",
            ))
        }
    }
}

#[derive(Debug)]
pub struct FlagPermission {
    pub read: bool,
    pub write: bool,
    pub clear: Option<usize>,
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

        let permission_ident = input.parse::<Ident>()?;
        let permissions = permission_ident.to_string().to_lowercase();

        if !permissions.chars().all(|c| matches!(c, 'r' | 'w' | 'c')) {
            return Err(syn::Error::new_spanned(
                &permission_ident,
                "expected permission string (e.g. `rw`, `r`, `wc(0)`)",
            ));
        }

        flag_permissions.read = permissions.contains("r");
        flag_permissions.write = permissions.contains("w");

        if permissions.contains('c') {
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
