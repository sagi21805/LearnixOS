mod bitfields;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

use crate::bitfields::BitFields;

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
