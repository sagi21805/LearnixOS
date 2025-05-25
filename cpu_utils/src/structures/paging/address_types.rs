use core::ptr::Alignment;

use crate::registers::cr3::{cr3_read, get_current_page_table};
#[cfg(target_arch = "x86_64")]
use constants::addresses::PHYSICAL_MEMORY_OFFSET;
use constants::enums::PageSize;

use super::page_tables::{PageTable, PageTableEntry};

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
            #[cfg(target_arch = "x86_64")]
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
            pub const unsafe fn as_mut_ptr<T>(&self) -> *mut T {
                self.0 as *mut T
            }

            #[inline]
            /// Return the underlying number as immutable pointer to data
            pub const fn as_ptr<T>(&self) -> *const T {
                self.0 as *const T
            }

            #[inline]
            /// Get the alignment of an address
            pub const fn alignment(&self) -> Alignment {
                unsafe { Alignment::new_unchecked(1 << self.0.trailing_zeros()) }
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct PhysicalAddress(pub usize);

impl_math_ops!(PhysicalAddress, usize);

impl_common_address_functions!(PhysicalAddress);

#[derive(Clone, Debug)]
pub struct VirtualAddress(pub usize);

impl_math_ops!(VirtualAddress, usize);

impl_common_address_functions!(VirtualAddress);

pub struct PageTableWalk {
    pub entries: [Option<&'static mut PageTableEntry>; 4],
    pub final_entry_index: usize,
}

impl VirtualAddress {
    #[allow(arithmetic_overflow)]
    pub const fn from_indexes(i4: usize, i3: usize, i2: usize, i1: usize) -> Self {
        Self((i4 << 39) | (i3 << 30) | (i2 << 21) | (i1 << 12) | 0)
    }
    /// indexing for the n_th page table
    ///
    /// 4 -> index of 4th table
    ///
    /// 3 -> index of 3rd table
    ///
    /// 2 -> index of 2nd table
    ///
    /// 1 -> index of 1st table
    pub const unsafe fn nth_pt_index_unchecked(&self, n: usize) -> usize {
        (self.0 >> (39 - 9 * (4 - n))) & 0o777
    }

    /// Reverse indexing for the address:
    ///
    /// 0 -> index of 4th table
    ///
    /// 1 -> index of 3rd table
    ///
    /// 2 -> index of 2nd table
    ///
    /// 3 -> index of 1st table
    #[allow(arithmetic_overflow)]
    pub const fn rev_nth_index_unchecked(&self, n: usize) -> usize {
        (self.0 >> (39 - (9 * n))) & 0o777
    }

    /// Get the relevant entries for a certain address, if they are not present a None type would replace them
    #[allow(unsafe_op_in_unsafe_fn)]
    #[cfg(target_arch = "x86_64")]
    pub fn walk(&self) -> PageTableWalk {
        let mut entries: [Option<&'static mut PageTableEntry>; 4] = [const { None }; 4];
        let mut final_entry_index = 0;
        let mut table: &'static mut PageTable = get_current_page_table();
        for i in 0..entries.len() {
            let table_index = self.rev_nth_index_unchecked(i);
            let entry_ptr = &mut table.entries[table_index] as *mut PageTableEntry;
            unsafe {
                if (*entry_ptr).present() {
                    entries[i] = Some(&mut *entry_ptr);
                    if !(*entry_ptr).huge_page() {
                        table = (*entry_ptr).as_table_mut_unchecked();
                    } else {
                        final_entry_index = i;
                        break;
                    }
                } else {
                    final_entry_index = i;
                    break;
                }
            }
        }
        PageTableWalk {
            entries,
            final_entry_index,
        }
    }
}

impl PhysicalAddress {
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub fn translate(&self) -> VirtualAddress {
        VirtualAddress(self.0 + PHYSICAL_MEMORY_OFFSET)
    }
}
