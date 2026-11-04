macro_rules! cpu_feature {
    ($flag_name:ident, $bit_number:expr) => {
        /// Checks if the feature is available
        ///
        /// `This method is auto-generated`
        #[inline]
        pub const fn ${concat(has_, $flag_name)}(&self) -> bool {
            self.0 & (1 << $bit_number) != 0
        }
    };
}

pub(crate) use cpu_feature;
