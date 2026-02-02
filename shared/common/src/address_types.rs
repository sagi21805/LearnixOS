use core::ptr::NonNull;

#[cfg(target_arch = "x86_64")]
use crate::constants::PHYSICAL_MEMORY_OFFSET;
use crate::enums::PageTableLevel;

// ANCHOR: trait_imports
use derive_more::{
    Add, AddAssign, AsMut, AsRef, Div, DivAssign, Mul, MulAssign, Sub,
    SubAssign,
};
use learnix_macros::CommonAddressFunctions;
// ANCHOR_END: trait_imports

// ANCHOR: physical_address
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
    CommonAddressFunctions,
)]
#[repr(C)]
pub struct PhysicalAddress(usize);

impl const From<usize> for PhysicalAddress {
    // TODO! Change into new in the future
    fn from(value: usize) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

// ANCHOR_END: physical_address

// ANCHOR: virtual_address
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
    CommonAddressFunctions,
)]
#[repr(C)]
pub struct VirtualAddress(usize);

impl<T> From<NonNull<T>> for VirtualAddress {
    fn from(value: NonNull<T>) -> Self {
        unsafe { VirtualAddress::new_unchecked(value.as_ptr().addr()) }
    }
}

impl const From<usize> for VirtualAddress {
    // TODO! Change into new in the future
    fn from(value: usize) -> Self {
        unsafe { Self::new_unchecked(value) }
    }
}

// ANCHOR_END: virtual_address

impl VirtualAddress {
    // ANCHOR: virtual_from_indexes
    #[allow(arithmetic_overflow)]
    pub const fn from_indexes(
        i4: usize,
        i3: usize,
        i2: usize,
        i1: usize,
    ) -> Self {
        Self((i4 << 39) | (i3 << 30) | (i2 << 21) | (i1 << 12))
    }
    // ANCHOR_END: virtual_from_indexes

    // ANCHOR: virtual_from_indices
    pub const fn from_indices(indices: [usize; 4]) -> Self {
        Self::from_indexes(indices[0], indices[1], indices[2], indices[3])
    }
    // ANCHOR_END: virtual_from_indices

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

    // pub fn translate(&self) -> Option<PhysicalAddress> {
    //     let mut current_table =
    // PageTable::current_table();     for i in 0..4 {
    //         let index = self.rev_nth_index_unchecked(i);
    //         match
    // current_table.entries[index].mapped_table_mut() {
    //             Ok(table) => current_table = table,
    //             Err(EntryError::NotATable) => {
    //                 return unsafe {
    // Some(current_table.entries[index].mapped_unchecked())
    // };             }
    //             Err(EntryError::NoMapping) => return
    // None,             Err(EntryError::Full) =>
    // unreachable!(),         }
    //     }
    //     None
    // }
}

impl PhysicalAddress {
    // ANCHOR: physical_translate
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub const fn translate(&self) -> VirtualAddress {
        VirtualAddress(self.0 + PHYSICAL_MEMORY_OFFSET)
    }
    // ANCHOR_END: physical_translate
}
