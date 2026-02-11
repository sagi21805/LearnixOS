macro_rules! cpu_feature {
    ($flag_name:ident, $bit_number:expr) => {
        /// Checks if the feature is available
        ///
        /// `This method is auto-generated`
        #[inline]
        #[allow(dead_code)]
        #[allow(unused_attributes)]
        pub const fn ${concat(has_, $flag_name)}(&self) -> bool {
            self.0 & (1 << $bit_number) != 0
        }
    };
}

pub(crate) use cpu_feature;
