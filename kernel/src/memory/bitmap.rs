use core::{ops::Index, slice};

use cpu_utils::structures::paging::address_types::{PhysicalAddress, VirtualAddress};
/// A low-level bitmap structure
///
/// # Safety
/// This structure directly operates on raw memory.
///
/// All methods marked `unsafe` must
/// be used with care, as they do not perform bounds checking or alignment validation.
///
/// This structure is working with a raw virtual address as the map entry
/// and this is the creator responsibility to make sure that this address is really unused
#[derive(Debug)]
pub struct BitMap {
    pub map: &'static mut [u64],
}

#[allow(unsafe_op_in_unsafe_fn)]
impl BitMap {
    /// Creates a new bitmap structure taking ownership on the map_address up to map_address + size
    ///
    /// All of the indexes will start with a default value of zero
    ///
    /// # Parameters
    ///
    /// - `map_address`: That address of the map which this structure will assume is owned by himself
    /// - `size`: The number of indexes the map array will have.
    ///           This will result in `size * 64` one bit indexes in the map itself
    ///
    /// # Safety
    ///
    /// The virtual address that is given to this structure is assumed to be owned by this structure.
    pub const unsafe fn new(map_address: PhysicalAddress, map_size: usize) -> BitMap {
        BitMap {
            map: slice::from_raw_parts_mut(map_address.as_mut_ptr::<u64>(), map_size),
        }
    }

    pub fn init(&mut self) {
        self.map.fill(0);
    }

    /// Set all of bit of the u64 into 1
    ///
    /// # Parameters
    ///
    /// - `map_index`: The index of the map array that will be set to 1
    /// # Safety
    ///
    /// Make sure the that map_index < self.size
    pub unsafe fn set_index_unchecked(&mut self, map_index: usize) {
        *self.map.get_unchecked_mut(map_index) = u64::MAX;
    }

    /// Return the number written in the map_index
    ///
    /// # Parameters
    ///
    /// - `map_index`: The value to index that map
    ///
    /// # Safety
    ///
    /// Make sure the that map_index < self.size
    pub unsafe fn get_index_unchecked(&self, map_index: usize) -> u64 {
        *self.map.get_unchecked(map_index)
    }

    /// Sets the bit corresponding to the `map_index` and the `bit_index`
    ///
    /// # Parameters
    ///
    /// - `map_index`: The index in the map to set the bit
    /// - `bit_index`: The index in the number we got from the `map_index` to set in the bit
    /// # Safety
    ///
    /// Make sure the that map_index < self.size and bit index < 64
    pub unsafe fn set_bit_unchecked(&mut self, map_index: usize, bit_index: u32) {
        *self.map.get_unchecked_mut(map_index) |= 1 << bit_index;
    }

    /// Returns the bit corresponding to the `map_index` and the `bit_index`
    ///
    /// # Parameters
    ///
    /// - `map_index`: The index in the map to get the bit
    /// - `bit_index`: The index in the number we got from the `map_index` to get in the bit
    ///
    /// # Safety
    ///
    /// Make sure the that map_index < self.size and bit index < 64
    pub unsafe fn get_bit_unchecked(&self, map_index: usize, bit_index: u64) -> bool {
        self.get_index_unchecked(map_index) & (1 << bit_index) != 0
    }

    pub fn as_slice(&self) -> &[u64] {
        self.map
    }
}
