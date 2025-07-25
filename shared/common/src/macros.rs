macro_rules! impl_common_address_functions {
    ($struct_name:ident) => {
        impl $struct_name {
            /// Create new instance from an address
            ///
            /// # Safety
            /// There is no check on the last bits of the address (bit 63-48 must be copies of bit 47)
            ///
            /// `This method is auto-generated`
            #[inline]
            pub const unsafe fn new_unchecked(address: usize) -> Self {
                Self(address)
            }

            /// Create new instance from an address, copies bit 47 to bits 63-48
            ///
            /// `This method is auto-generated`
            #[inline]
            #[cfg(target_arch = "x86_64")]
            pub const fn new(address: usize) -> Self {
                Self((address << 16) >> 16)
            }

            #[inline]
            /// Return the underlying usize
            pub const fn as_usize(&self) -> usize {
                self.0
            }

            #[inline]
            /// Return the underlying number as mutable pointer to data
            ///
            /// # Safety
            /// This method returns a mutable pointer without checking if this address is used or not
            pub const unsafe fn as_mut_ptr<T>(&self) -> *mut T {
                self.0 as *mut T
            }

            #[inline]
            /// Return the underlying number as immutable pointer to data
            pub const fn as_ptr<T>(&self) -> *const T {
                self.0 as *const T
            }

            #[inline]
            /// Checks if this address is aligned to a certain alignment
            pub const fn is_aligned(&self, alignment: core::ptr::Alignment) -> bool {
                self.0 & (alignment.as_usize() - 1) == 0
            }

            #[inline]
            pub const fn align_up(mut self, alignment: core::ptr::Alignment) -> Self {
                self.0 = (self.0 + (alignment.as_usize() - 1)) & !(alignment.as_usize() - 1);
                self
            }

            #[inline]
            pub const fn align_down(mut self, alignment: core::ptr::Alignment) -> Self {
                self.0 &= !(alignment.as_usize() - 1);
                self
            }

            #[inline]
            /// Get the alignment of an address
            pub const fn alignment(&self) -> core::ptr::Alignment {
                unsafe { core::ptr::Alignment::new_unchecked(1 << self.0.trailing_zeros()) }
            }
        }
    };
}

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
        pub const fn $flag_name(self) -> Self {
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
        pub const fn ${concat(is_, $flag_name)}(&self) -> bool {
            self.0 & (1 << $bit_number) != 0
        }
    };
}
