#[macro_export]
macro_rules! flag {
    ($bit_name:ident, $bit_number:literal) => {
        // Create a setter function like set_bit_name
        #[inline]
        pub const fn ${concat(set_, $bit_name)}(&mut self) {
            self.0 |= 1 << $bit_number;
        }

        #[inline]
        pub const fn ${concat(set_chain_, $bit_name)}(self) -> Self {
            Self(self.0 | (1 << $bit_number))
        }
        // Create a constant function for setting the bit
        #[inline]
        pub const fn $bit_name(&self) -> bool {
            self.0 & (1 << $bit_number) != 0
        }
    };
}

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
