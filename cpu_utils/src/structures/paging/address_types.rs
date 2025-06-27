use core::ptr::Alignment;

use super::page_tables::{PageTable, PageTableEntry};
#[cfg(target_arch = "x86_64")]
use common::constants::addresses::PHYSICAL_MEMORY_OFFSET;
use derive_more::{
    Add, AddAssign, AsMut, AsRef, Div, DivAssign, From, Mul, MulAssign, Sub, SubAssign,
};

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
            pub const fn alignment(&self) -> Alignment {
                unsafe { Alignment::new_unchecked(1 << self.0.trailing_zeros()) }
            }
        }
    };
}

#[derive(
    Clone,
    Debug,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Default,
    AsMut,
    AsRef,
    From,
)]
pub struct PhysicalAddress(pub usize);

impl_common_address_functions!(PhysicalAddress);

#[derive(
    Clone,
    Debug,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Default,
    AsMut,
    AsRef,
    From,
)]
pub struct VirtualAddress(pub usize);

impl_common_address_functions!(VirtualAddress);

pub struct PageTableWalk {
    pub entries: [Option<&'static mut PageTableEntry>; 4],
    pub final_entry_index: usize,
}

impl VirtualAddress {
    /// Constructs a virtual address from four page table indices.
    ///
    /// The indices correspond to the four levels of the x86_64 page table hierarchy, with `i4` as the highest level (P4) and `i1` as the lowest (P1).
    ///
    /// # Parameters
    /// - `i4`: Index for the P4 (top-level) page table.
    /// - `i3`: Index for the P3 page table.
    /// - `i2`: Index for the P2 page table.
    /// - `i1`: Index for the P1 (lowest-level) page table.
    ///
    /// # Returns
    /// A `VirtualAddress` representing the combined address from the provided indices.
    ///
    /// # Examples
    ///
    /// ```
    /// let addr = VirtualAddress::from_indexes(0x1, 0x2, 0x3, 0x4);
    /// assert_eq!(addr.as_usize(), (0x1 << 39) | (0x2 << 30) | (0x3 << 21) | (0x4 << 12));
    /// ```
    pub const fn from_indexes(i4: usize, i3: usize, i2: usize, i1: usize) -> Self {
        Self((i4 << 39) | (i3 << 30) | (i2 << 21) | (i1 << 12) | 0)
    }

    /// Constructs a virtual address from an array of four page table indices.
    ///
    /// The indices correspond to the four levels of the x86_64 page table hierarchy, ordered from highest (P4) to lowest (P1).
    ///
    /// # Examples
    ///
    /// ```
    /// let indices = [0x1, 0x2, 0x3, 0x4];
    /// let addr = VirtualAddress::from_indices(indices);
    /// ```
    pub const fn from_indices(indices: [usize; 4]) -> Self {
        Self::from_indexes(indices[0], indices[1], indices[2], indices[3])
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

    /// Returns the page table index at the specified reverse level for this virtual address.
    ///
    /// Level 0 corresponds to the 4th-level table (PML4), level 1 to the 3rd-level table (PDPT), level 2 to the 2nd-level table (PD), and level 3 to the 1st-level table (PT).
    ///
    /// # Safety
    ///
    /// No bounds checking is performed on `n`; passing a value outside 0..=3 may yield meaningless results.
    ///
    /// # Examples
    ///
    /// ```
    /// let addr = VirtualAddress::from_indexes(1, 2, 3, 4);
    /// assert_eq!(addr.rev_nth_index_unchecked(0), 1); // 4th-level index
    /// assert_eq!(addr.rev_nth_index_unchecked(3), 4); // 1st-level index
    /// ```
    #[allow(arithmetic_overflow)]
    pub const fn rev_nth_index_unchecked(&self, n: usize) -> usize {
        (self.0 >> (39 - (9 * n))) & 0o777
    }
}

impl PhysicalAddress {
    /// Converts this physical address to a virtual address using the physical memory offset on x86_64.
    ///
    /// # Returns
    ///
    /// A `VirtualAddress` corresponding to this physical address mapped with the configured physical memory offset.
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub const fn translate(&self) -> VirtualAddress {
        VirtualAddress(self.0 + PHYSICAL_MEMORY_OFFSET)
    }
}
