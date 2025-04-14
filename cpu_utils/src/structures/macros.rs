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

/// This macro will generate simple mathematical operations for wrapping structs
///
/// ```rust
/// struct Example(u32);
///
/// impl_math_ops!(Example, u32)
/// ```
///
/// The following code example will generate the following methods for the Example struct
///
/// `add`,
/// `add_assign`,
/// `sub`,
/// `sub_assign`,
/// `mul`,
/// `mul_assign`,
/// `div`,
/// `div_assign`

#[macro_export]
macro_rules! impl_math_ops {
    ($struct_name:ident, $inner_type:ty) => {
        impl core::ops::Add<$inner_type> for $struct_name {
            type Output = Self;
            fn add(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 + rhs)
            }
        }

        impl core::ops::AddAssign<$inner_type> for $struct_name {
            fn add_assign(&mut self, rhs: $inner_type) {
                self.0 += rhs;
            }
        }

        impl core::ops::Sub<$inner_type> for $struct_name {
            type Output = Self;
            fn sub(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 - rhs)
            }
        }

        impl core::ops::SubAssign<$inner_type> for $struct_name {
            fn sub_assign(&mut self, rhs: $inner_type) {
                self.0 -= rhs;
            }
        }

        impl core::ops::Mul<$inner_type> for $struct_name {
            type Output = Self;
            fn mul(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl core::ops::MulAssign<$inner_type> for $struct_name {
            fn mul_assign(&mut self, rhs: $inner_type) {
                self.0 *= rhs;
            }
        }

        impl core::ops::Div<$inner_type> for $struct_name {
            type Output = Self;
            fn div(self, rhs: $inner_type) -> Self::Output {
                Self(self.0 / rhs)
            }
        }

        impl core::ops::DivAssign<$inner_type> for $struct_name {
            fn div_assign(&mut self, rhs: $inner_type) {
                self.0 /= rhs;
            }
        }
    };
}

#[macro_export]
macro_rules! impl_common_address_functions {
    ($struct_name:ident) => {
        impl $struct_name {
            /// Create new instance from an address
            ///
            /// # Safety
            /// There is no check on the last bits of the address (bit 63-48 must be copies of bit 47)
            ///
            /// `This method is auto-generated`
            pub const unsafe fn new_unchecked(address: usize) -> Self {
                Self(address)
            }

            /// Create new instance from an address, copies bit 47 to bits 63-48
            ///
            /// `This method is auto-generated`
            pub const fn new(address: usize) -> Self {
                Self((address << 16) >> 16)
            }

            /// Create new instance with the zero address
            pub const fn zero() -> Self {
                Self(0)
            }

            #[inline]
            /// Return the underlying usize
            pub const fn as_usize(&self) -> usize {
                self.0
            }

            #[inline]
            /// Checks if this address is aligned to a certain alignment
            pub const fn is_aligned(&self, alignment: core::ptr::Alignment) -> bool {
                self.0 & (alignment.as_usize() - 1) == 0
            }

            #[inline]
            /// Return the underlying number as mutable pointer to data
            ///
            /// # Safety
            /// This method returns a mutable pointer without checking if this address is used or not
            pub const unsafe fn as_ptr_mut(&self) -> *mut u8 {
                self.0 as *mut u8
            }

            #[inline]
            /// Return the underlying number as immutable pointer to data
            pub const fn as_ptr(&self) -> *const u8 {
                self.0 as *const u8
            }
        }
    };
}
