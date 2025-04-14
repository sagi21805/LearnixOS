/// This is a simple implementation of a physical and virtual addresses.
/// 
/// This interface is mainly used to provide explicit meaning for when a physical address is needed and when a virtual one
/// 
/// This module implements some very basic commands for the addresses and more rich and useful interface will be added in the kernel 
/// -------------------------------------------------------------------------------------------------------------

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

trait Address { }

#[derive(Clone, Debug)]
pub struct PhysicalAddress(usize);

impl_math_ops!(PhysicalAddress, usize);

impl_common_address_functions!(PhysicalAddress);

#[derive(Clone, Debug)]
pub struct VirtualAddress(usize);

impl_math_ops!(VirtualAddress, usize);

impl_common_address_functions!(VirtualAddress);


impl VirtualAddress {


    pub fn translate(&self) -> PhysicalAddress {
        todo!()
    }

    // Bits 48-39
    #[allow(arithmetic_overflow)]
    pub const fn pt4_index(&self) -> usize {
        (self.0 >> 39) & 0o777
    }
    // Bit 39-30
    pub const fn pt3_index(&self) -> usize {
        (self.0 >> 30) & 0o777
    }
    // Bits 30-21
    pub const fn pt2_index(&self) -> usize {
        (self.0 >> 21) & 0o777
    }
    // Bits 21-12
    pub const fn pt1_index(&self) -> usize {
        (self.0 >> 12) & 0o777
    }
    // index of the n_th page table
    pub const fn nth_pt_index(&self, n: usize) -> usize {
        if n > 4 || n < 1 {
            panic!("There are only 4 page tables, you tried to index table out of range");
        }
        (self.0 >> (39 - 9 * (4 - n))) & 0o777
    }
}
