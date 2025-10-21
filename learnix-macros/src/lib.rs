use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

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
                self.0 as *mut T
            }
            pub const fn as_ptr<T>(&self) -> *const T {
                self.0 as *const T
            }
            pub const fn is_aligned(&self, alignment: core::ptr::Alignment) -> bool {
                self.0 & (alignment.as_usize() - 1) == 0
            }
            pub const fn align_up(mut self, alignment: core::ptr::Alignment) -> Self {
                self.0 = (self.0 + (alignment.as_usize() - 1)) & !(alignment.as_usize() - 1);
                self
            }
            pub const fn align_down(mut self, alignment: core::ptr::Alignment) -> Self {
                self.0 &= !(alignment.as_usize() - 1);
                self
            }
            pub const fn alignment(&self) -> core::ptr::Alignment {
                unsafe { core::ptr::Alignment::new_unchecked(1 << self.0.trailing_zeros()) }
            }
        }
    };

    expanded.into()
}
