use flag::FlagInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

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
            pub const unsafe fn as_mut_ptr<T>(&self) -> *mut T {
                core::ptr::with_exposed_provenance_mut::<T>(self.0)
            }
            pub const fn as_ptr<T>(&self) -> *const T {
                core::ptr::with_exposed_provenance::<T>(self.0)
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
                    core::ptr::Alignment::new_unchecked(
                        1 << self.0.trailing_zeros(),
                    )
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
        pub const fn #set_ident(&mut self) {
            self.0 |= 1 << #bit;
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
        pub const fn #unset_ident(&mut self) {
            self.0 &= !(1 << #bit);
        }

        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Checks if the corresponding flag is set
        pub const fn #is_ident(&self) -> bool {
            (self.0 & (1 << #bit)) != 0
        }
    };

    expanded.into()
}
// ANCHOR_END: flag
