#[macro_export]
/// This macro will obtain `flag_name` and the corresponding `bit_number`
///
/// With this information it will automatically generate three methods
///
/// 1. `set_<flag_name>`: will set the bit without returning self
/// 2. `set_chain_<flag_name>`: will set the bit and will return self
/// 3. `<flag_name>`: will read the flag and return true if the flag is set or false if not
macro_rules! flag {
    ($flag_name:ident, $bit_number:literal) => {
        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Sets the corresponding flag
        ///
        /// `This method is auto-generated`
        pub const fn ${concat(set_, $flag_name)}(&mut self) {
            self.0 |= 1 << $bit_number;
        }

        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Sets the corresponding flag while returning self
        ///
        /// `This method is auto-generated`
        pub const fn ${concat(set_chain_, $flag_name)}(self) -> Self {
            Self(self.0 | (1 << $bit_number))
        }

        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        /// Unset the corresponding flag
        ///
        /// `This method is auto-generated`
        pub const fn ${concat(unset_, $flag_name)}(&mut self) {
            self.0 &= !(1 << $bit_number)
        }

        /// Checks if the corresponding flag in set to 1
        ///
        /// `This method is auto-generated`
        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        pub const fn $flag_name(&self) -> bool {
            self.0 & (1 << $bit_number) != 0
        }
    };
}
