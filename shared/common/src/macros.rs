// ANCHOR: page_flag_macro
macro_rules! page_flag {
    ($bit_number:literal, $zero_name:ident, $one_name:ident) => {
        pub const fn $zero_name(&self) -> bool {
            self.0 & (1 << $bit_number) == 0
        }

        pub const fn $one_name(&self) -> bool {
            self.0 & (1 << $bit_number) == 1
        }
    };
}
// ANCHOR_END: page_flag_macro
