use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, LitInt, Token};

pub struct FlagInput {
    pub name: Ident,
    pub _comma: Token![,],
    pub bit: LitInt,
}

impl Parse for FlagInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let bit: LitInt = input.parse()?;
        Ok(FlagInput { name, _comma, bit })
    }
}
