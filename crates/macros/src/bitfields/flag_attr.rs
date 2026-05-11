use syn::{
    Ident, LitInt, Token, TypePath,
    parse::{Parse, discouraged::Speculative},
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

impl Parse for FlagAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attributes = FlagAttribute::default();

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
                    Ok(_kw) => {
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

        let permission =
            input.parse::<Ident>()?.to_string().to_lowercase();

        if !permission.chars().all(|c| matches!(c, 'r' | 'w' | 'c')) {
            return Err(syn::Error::new_spanned(
                &permission,
                "expected permission string (e.g. `rw`, `r`, `wc(0)`)",
            ));
        }

        flag_permissions.read = permission.contains("r");
        flag_permissions.write = permission.contains("w");

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
