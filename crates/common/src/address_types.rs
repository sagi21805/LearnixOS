use core::ptr::NonNull;

#[cfg(target_arch = "x86_64")]
use crate::constants::PHYSICAL_MEMORY_OFFSET;
use crate::enums::PageTableLevel;

use derive_more::{
    Add, AddAssign, AsMut, AsRef, Div, DivAssign, Mul, MulAssign, Sub,
    SubAssign,
};

pub const trait CommonAddressFunctions:
    Sized + Clone + Copy
{
    /// Create new instance without checking for address alignment.
    ///
    /// # Safety
    /// This function should not check for sign extension.
    unsafe fn new_unchecked(address: usize) -> Self;

    fn new(address: usize) -> Option<Self>;

    fn as_usize(&self) -> usize;

    fn as_non_null<T>(&self) -> core::ptr::NonNull<T> {
        core::ptr::NonNull::new(
            core::ptr::with_exposed_provenance_mut::<T>(self.as_usize()),
        )
        .expect("Tried to create NonNull from address, found null")
    }

    fn is_aligned(&self, alignment: core::ptr::Alignment) -> bool {
        self.as_usize() & (alignment.as_usize() - 1) == 0
    }

    fn align_up(self, alignment: core::ptr::Alignment) -> Self {
        unsafe {
            Self::new_unchecked(
                (self.as_usize() + (alignment.as_usize() - 1))
                    & !(alignment.as_usize() - 1),
            )
        }
    }

    fn align_down(self, alignment: core::ptr::Alignment) -> Self {
        unsafe {
            Self::new_unchecked(
                self.as_usize() & !(alignment.as_usize() - 1),
            )
        }
    }

    fn alignment(&self) -> core::ptr::Alignment {
        unsafe {
            if self.as_usize() == 0 {
                // Address 0 is aligned to any alignment; return max
                // representable.
                core::ptr::Alignment::new_unchecked(1 << (usize::BITS - 1))
            } else {
                core::ptr::Alignment::new_unchecked(
                    1 << self.as_usize().trailing_zeros(),
                )
            }
        }
    }
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
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[repr(C)]
pub struct PhysicalAddress(usize);

impl const CommonAddressFunctions for PhysicalAddress {
    unsafe fn new_unchecked(address: usize) -> Self {
        Self(address)
    }

    fn new(address: usize) -> Option<Self> {
        #[cfg(target_arch = "x86_64")]
        if address < (1 << 48) {
            unsafe { Some(Self::new_unchecked(address)) }
        } else {
            None
        }

        #[cfg(target_arch = "x86")]
        unsafe {
            Some(Self::new_unchecked(address))
        }
    }

    fn as_usize(&self) -> usize {
        self.0
    }
}

impl const From<usize> for PhysicalAddress {
    fn from(value: usize) -> Self {
        unsafe { PhysicalAddress::new_unchecked(value) }
    }
}

impl const From<u64> for PhysicalAddress {
    fn from(value: u64) -> Self {
        unsafe { PhysicalAddress::new_unchecked(value as usize) }
    }
}

impl const From<PhysicalAddress> for u64 {
    fn from(value: PhysicalAddress) -> Self {
        value.0 as u64
    }
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
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[repr(C)]
pub struct VirtualAddress(usize);

impl const CommonAddressFunctions for VirtualAddress {
    unsafe fn new_unchecked(address: usize) -> Self {
        Self(address)
    }

    fn new(address: usize) -> Option<Self> {
        #[cfg(target_arch = "x86_64")]
        if address < (1usize << 48) {
            return Some(unsafe {
                Self::new_unchecked(
                    (((address << 16) as isize) >> 16) as usize,
                )
            });
        } else {
            None
        }

        #[cfg(target_arch = "x86")]
        unsafe {
            Some(Self::new_unchecked(address))
        }
    }

    fn as_usize(&self) -> usize {
        self.0
    }
}

impl<T> From<NonNull<T>> for VirtualAddress {
    fn from(value: NonNull<T>) -> Self {
        unsafe { VirtualAddress::new_unchecked(value.as_ptr().addr()) }
    }
}

impl const From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        unsafe { VirtualAddress::new_unchecked(value) }
    }
}

impl VirtualAddress {
    #[allow(arithmetic_overflow)]
    pub const fn from_indexes(
        i4: usize,
        i3: usize,
        i2: usize,
        i1: usize,
    ) -> Self {
        Self((i4 << 39) | (i3 << 30) | (i2 << 21) | (i1 << 12))
    }

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
    // ANCHOR: virtual_nth_pt_index_unchecked
    pub const fn index_of(&self, level: PageTableLevel) -> usize {
        (self.0 >> (39 - 9 * (level as usize))) & 0o777
    }
}

impl PhysicalAddress {
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub const fn translate(&self) -> VirtualAddress {
        VirtualAddress(self.0 + PHYSICAL_MEMORY_OFFSET)
    }
}
