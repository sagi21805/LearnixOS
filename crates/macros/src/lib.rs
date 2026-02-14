use flag::FlagInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Block, DeriveInput, FieldMutability, FnArg, Ident, ItemFn, ItemStruct,
    LitInt, ReturnType, Signature, Token, parse_macro_input,
    punctuated::Punctuated, token::Token,
};

use crate::bitflags::bitfields_impl;

mod bitflags;
mod flag;
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

// ANCHOR: flag
/// This macro will obtain `flag_name` and the corresponding
/// `bit_number`
///
/// With this information it will automatically generate
/// three methods
///
/// 1. `set_$flag_name`: set the bit without returning self
/// 2. `$flag_name`: set the bit and will return self
/// 3. `unset_$flag_name`: unset the bit without returning self
/// 4. `is_$flag_name`: return true if the flag is set or false if not
#[proc_macro]
pub fn flag(input: TokenStream) -> TokenStream {
    let FlagInput { name, bit, .. } =
        syn::parse_macro_input!(input as FlagInput);

    // build identifiers
    let name_str = name.to_string();
    let set_ident = format_ident!("set_{}", name_str);
    let unset_ident = format_ident!("unset_{}", name_str);
    let is_ident = format_ident!("is_{}", name_str);

    let expanded = quote! {
        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Sets the corresponding flag
        pub fn #set_ident(&mut self) {
            unsafe {
                let val = core::ptr::read_volatile(
                    self as *const _ as *mut usize
                );

                core::ptr::write_volatile(
                    self as *const _ as *mut usize,
                    val | (1 << #bit) as usize
                )
            }
        }

        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Sets the corresponding flag while returning self
        pub const fn #name(self) -> Self {
            Self(self.0 | (1 << #bit))
        }

        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Unset the corresponding flag
        pub fn #unset_ident(&mut self) {
            unsafe {
                let val = core::ptr::read_volatile(
                    self as *const _ as *mut usize
                );

                core::ptr::write_volatile(
                    self as *const _ as *mut usize,
                    val & !(1 << #bit) as usize
                )
            }
        }

        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Checks if the corresponding flag is set
        pub fn #is_ident(&self) -> bool {
            unsafe {
                core::ptr::read_volatile(
                    self as *const _ as *mut usize
                ) & ((1<< #bit) as usize) != 0
            }
        }
    };

    expanded.into()
}
// ANCHOR_END: flag

// ANCHOR: ro_flag
/// This macro will obtain `flag_name` and the corresponding
/// `bit_number` and create read-only flag functionality
///
/// With this information it will automatically generate
/// three methods
///
/// 1. `is_$flag_name`: return true if the flag is set or false if not
#[proc_macro]
pub fn ro_flag(input: TokenStream) -> TokenStream {
    let FlagInput { name, bit, .. } =
        syn::parse_macro_input!(input as FlagInput);

    // build identifiers
    let name_str = name.to_string();
    let support_ident = format_ident!("is_{}", name_str);

    let expanded = quote! {
        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Checks if the corresponding flag is set
        pub fn #support_ident(&self) -> bool {
            unsafe {
                core::ptr::read_volatile(
                    self as *const _ as *mut usize
                ) & ((1<< #bit) as usize) != 0
            }
        }
    };

    expanded.into()
}
// ANCHOR_END: ro_flag

// ANCHOR: rwc_flag
#[proc_macro]
pub fn rwc_flag(input: TokenStream) -> TokenStream {
    let FlagInput { name, bit, .. } =
        syn::parse_macro_input!(input as FlagInput);

    // build identifiers
    let name_str = name.to_string();
    let clear_ident = format_ident!("clear_{}", name_str);
    let support_ident = format_ident!("is_{}", name_str);

    let expanded = quote! {
        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Sets the corresponding flag
        pub const fn #clear_ident(&mut self) {
            self.0 |= 1 << #bit;
        }


        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Checks if the corresponding flag is set
        pub fn #support_ident(&self) -> bool {
            unsafe {
                core::ptr::read_volatile(
                    self as *const _ as *mut usize
                ) & ((1<< #bit) as usize) != 0
            }
        }
    };

    expanded.into()
}
// ANCHOR_END: rwc_flag

// ANCHOR: rw1_flag
#[proc_macro]
pub fn rw1_flag(input: TokenStream) -> TokenStream {
    let FlagInput { name, bit, .. } =
        syn::parse_macro_input!(input as FlagInput);

    // build identifiers
    let name_str = name.to_string();
    let set_ident = format_ident!("set_{}", name_str);

    let expanded = quote! {
        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Sets the corresponding flag
        pub const fn #set_ident(&mut self) {
            self.0 |= 1 << #bit;
        }
    };

    expanded.into()
}
// ANCHOR_END: rw1_flag

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
pub fn bitfields(attr: TokenStream, item: TokenStream) -> TokenStream {
    let s = parse_macro_input!(item as ItemStruct);
    bitfields_impl(s).into()
}
