use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, ItemStruct, LitInt, Token, parse_macro_input,
    punctuated::Punctuated,
};

use crate::bitflags::bitfields_impl;

mod bitflags;
// ANCHOR: common_address_functions
#[proc_macro_derive(CommonAddressFunctions)]
pub fn common_address_functions(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let expanded = quote! {
        impl #struct_name {
            pub const unsafe fn new_unchecked(address: usize) -> Self {
                Self(address)
            }
            pub const fn as_usize(&self) -> usize {
                self.0
            }
            pub const fn as_non_null<T>(&self) -> core::ptr::NonNull<T> {
                core::ptr::NonNull::new(
                    core::ptr::with_exposed_provenance_mut::<T>(self.0)
                ).expect("Tried to create NonNull from address, found null")
            }
            pub const fn is_aligned(
                &self,
                alignment: core::ptr::Alignment,
            ) -> bool {
                self.0 & (alignment.as_usize() - 1) == 0
            }
            pub const fn align_up(
                mut self,
                alignment: core::ptr::Alignment,
            ) -> Self {
                self.0 = (self.0 + (alignment.as_usize() - 1))
                    & !(alignment.as_usize() - 1);
                self
            }
            pub const fn align_down(
                mut self,
                alignment: core::ptr::Alignment,
            ) -> Self {
                self.0 &= !(alignment.as_usize() - 1);
                self
            }
            pub const fn alignment(&self) -> core::ptr::Alignment {
                unsafe {
                    if self.0 == 0 {
                        // Address 0 is aligned to any alignment; return max representable.
                        core::ptr::Alignment::new_unchecked(1 << (usize::BITS - 1))
                    } else {
                        core::ptr::Alignment::new_unchecked(1 << self.0.trailing_zeros())
                    }
                }
            }
        }
    };

    expanded.into()
}
// ANCHOR_END: common_address_functions

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
pub fn bitfields(attr: TokenStream, item: TokenStream) -> TokenStream {
    let s = parse_macro_input!(item as ItemStruct);
    bitfields_impl(s)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
