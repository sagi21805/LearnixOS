mod bitfields;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    ItemStruct, LitInt, Token, parse_macro_input, punctuated::Punctuated,
};

use crate::bitfields::BitFields;

#[proc_macro]
pub fn generate_generics(input: TokenStream) -> TokenStream {
    // Parse the input as a comma-separated list of integers: 8, 16, 32...
    let parser = Punctuated::<LitInt, Token![,]>::parse_terminated;
    let input = parse_macro_input!(input with parser);

    let mut expanded = quote! {};

    // initial range for the first item
    let mut last_size: usize = 0;

    for lit in input {
        let generic_size: usize = lit
            .base10_parse()
            .expect("Invalid integer format, expected base10");

        let generic_name = format_ident!("Generic{}", generic_size);

        // minimum size of 8 bytes (usize on 64 bit).
        let array_size = generic_size / 8;

        let start = last_size;
        let end = generic_size;

        let struct_def = quote! {
            #[derive(Debug, Clone, Copy)]
            pub struct #generic_name(pub [usize; #array_size]);

            impl Generic for #generic_name {
                fn size(&self) -> usize { #generic_size }
                const START: usize = #start;
                const END: usize = #end;
            }
        };

        last_size = generic_size + 1;
        expanded.extend(struct_def);
    }

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
/// Turn a struct into a bitfield struct.
///
/// ```rust
/// struct MyBitfield {
///
///     #[rwc(10)] // read-write-clearable flag with clear value of 10
///     flag1: B1,  // 1 bit field
///
///     #[flag(r)] // Read-only flag
///     flag2: B3,  // 3 bits field
///     flag3: B10, // 10 bits field
/// }
///
/// let b = MyBitField::new();
///
/// b.set_flag1(1);
/// b.set_flag3(20);
///
/// assert_eq!(b.get_flag3(), 20);
/// assert_eq!(size_of::<MyBitField>(), size_of::<u16>());
/// ```
pub fn bitfields(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let s = parse_macro_input!(item as ItemStruct);
    BitFields::try_from(&s)
        .map(|bitfields| quote! {#bitfields})
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
