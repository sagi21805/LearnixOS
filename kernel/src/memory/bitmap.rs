use core::{slice, u64};
use cpu_utils::structures::paging::address_types::VirtualAddress;
use derive_more::Constructor;

use crate::println;

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub map_index: usize,
    pub bit_index: usize,
}
impl Position {
    /// Creates a new `Position` with the given map and bit indices if the bit index is within bounds.
    ///
    /// Returns `None` if `bit_index` is 64 or greater, as it would exceed the bounds of a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// let pos = Position::new(2, 10);
    /// assert!(pos.is_some());
    ///
    /// let invalid = Position::new(1, 64);
    /// assert!(invalid.is_none());
    /// ```
    pub fn new(map_index: usize, bit_index: usize) -> Option<Self> {
        if bit_index < 64 {
            Some(Position {
                map_index,
                bit_index,
            });
        }
        None
    }

    /// Creates a `Position` from the given map and bit indices without bounds checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bit_index < 64`, as no validation is performed.
    ///
    /// # Examples
    ///
    /// ```
    /// let pos = unsafe { Position::new_unchecked(2, 10) };
    /// assert_eq!(pos.map_index, 2);
    /// assert_eq!(pos.bit_index, 10);
    /// ```
    pub const unsafe fn new_unchecked(map_index: usize, bit_index: usize) -> Position {
        Position {
            map_index,
            bit_index,
        }
    }

    /// Creates a `Position` from an absolute bit index within the bitmap.
    ///
    /// Converts a global bit index into a `Position` by determining the corresponding `map_index` and `bit_index`.
    ///
    /// # Examples
    ///
    /// ```
    /// let pos = Position::from_abs_bit_index(130);
    /// assert_eq!(pos.map_index, 2);
    /// assert_eq!(pos.bit_index, 2);
    /// ```
    pub const fn from_abs_bit_index(bit_index: usize) -> Self {
        unsafe {
            Self::new_unchecked(
                bit_index / u64::BITS as usize,
                bit_index % u64::BITS as usize,
            )
        }
    }
}

#[derive(Debug, Clone, Default, Constructor)]
pub struct ContiguousBlockLayout {
    pub low_mask: u64,
    pub index_count: usize,
    pub high_mask: u64,
}

impl ContiguousBlockLayout {
    /// Creates an initial contiguous block layout and end position for a block of the given bit size.
    ///
    /// Returns a `ContiguousBlockLayout` representing the masks and index count needed to cover `size_bits` bits starting at bit index 0, along with the `Position` marking the end of the block.
    ///
    /// # Parameters
    /// - `size_bits`: The number of contiguous bits to cover.
    ///
    /// # Returns
    /// A tuple containing the block layout and the end position after the block.
    ///
    /// # Examples
    ///
    /// ```
    /// let (layout, end_pos) = ContiguousBlockLayout::initial_from_bits(100);
    /// assert!(layout.low_mask != 0);
    /// ```
    pub(self) fn initial_from_bits(size_bits: usize) -> (Self, Position) {
        let remain_after_low = size_bits.saturating_sub(u64::BITS as usize);
        let low_bit_count = size_bits - remain_after_low;
        let remain_after_high = remain_after_low.saturating_sub(u64::BITS as usize);
        let high_bit_count = remain_after_low - remain_after_high;
        let index_count = remain_after_high / u64::BITS as usize;
        (
            Self::new(
                u64::MAX.unbounded_shr(u64::BITS - low_bit_count as u32),
                index_count,
                u64::MAX.unbounded_shr(u64::BITS - high_bit_count as u32),
            ),
            unsafe {
                Position::new_unchecked(
                    index_count + (low_bit_count / u64::MAX as usize),
                    high_bit_count,
                )
            },
        )
    }

    /// Constructs a `ContiguousBlockLayout` representing a block of bits starting at a given position and spanning a specified number of bits.
    ///
    /// Calculates the appropriate masks and index count to cover the block, accounting for alignment within the starting `u64`.
    ///
    /// # Parameters
    ///
    /// - `p`: The starting position within the bitmap.
    /// - `size_bits`: The number of bits in the block.
    ///
    /// # Returns
    ///
    /// A `ContiguousBlockLayout` describing the layout of the contiguous block.
    ///
    /// # Examples
    ///
    /// ```
    /// let start = Position::new(0, 10).unwrap();
    /// let layout = ContiguousBlockLayout::from_start_size(&start, 20);
    /// assert!(layout.low_mask != 0);
    /// ```
    pub fn from_start_size(p: &Position, size_bits: usize) -> Self {
        let remain_after_low = size_bits.saturating_sub(u64::BITS as usize - p.bit_index);
        let low_bit_count = size_bits - remain_after_low;
        let index_count = remain_after_low / u64::BITS as usize;
        let high_bit_count = remain_after_low & (u64::BITS as usize - 1);
        Self::new(
            (u64::MAX.unbounded_shr(u64::BITS - low_bit_count as u32)) << p.bit_index,
            index_count,
            u64::MAX.unbounded_shr(u64::BITS - high_bit_count as u32),
        )
    }

    /// Resets the block layout to its initial state, assuming `low_mask` is zero.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `self.low_mask` is zero before calling this method. Violating this precondition may result in incorrect block layout state.
    fn reset_unchecked(&mut self) {
        let is_zero = self.index_count == 0;
        self.index_count -= is_zero as usize;
        self.low_mask = self.high_mask | (u64::MAX + !is_zero as u64);
        self.high_mask &= u64::MAX + !is_zero as u64;
    }

    /// Shifts the contiguous block layout left by one bit, updating masks and index count.
    ///
    /// If the low mask overflows, the overflow is added to the high mask. When the low mask becomes zero, the layout is reset. Returns the value used to adjust the high mask and index count after the shift.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut layout = ContiguousBlockLayout::from_start_size(&Position::new(0, 0).unwrap(), 70);
    /// let val = layout.shift();
    /// assert!(val == 0 || val == 1);
    /// ```
    pub(self) fn shift(&mut self) -> u64 {
        let (new_low_mask, overlowed) = self.low_mask.overflowing_shl(1);
        self.low_mask = new_low_mask;
        self.high_mask += overlowed as u64;
        self.high_mask <<= 1;
        let val = self.high_mask.saturating_sub(u64::MAX - 1) as u64;
        self.high_mask += val;
        self.index_count += val as usize;
        if self.low_mask == 0 {
            self.reset_unchecked();
        }
        val
    }
}

/// A low-level bitmap structure
#[derive(Debug)]
pub struct BitMap {
    pub map: &'static mut [u64],
}

impl core::ops::Index<usize> for BitMap {
    type Output = u64;

    /// Returns a reference to the u64 value at the specified index in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 4);
    /// let value = bitmap.index(2);
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        &self.map[index]
    }
}

impl core::ops::IndexMut<usize> for BitMap {
    /// Returns a mutable reference to the u64 value at the specified index in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(addr, size);
    /// let value = bitmap.index_mut(0);
    /// *value = 0xFFFF_FFFF_FFFF_FFFF;
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.map[index]
    }
}

impl BitMap {
    /// Creates a new bitmap over a memory region, assuming ownership of the specified address and size.
    ///
    /// The bitmap will cover `map_size * 64` bits, with all bits initially set to zero. The memory at `map_address` must be valid for a mutable slice of `map_size` `u64` values and is assumed to be exclusively owned by the bitmap for its lifetime.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `map_address` points to a valid, writable memory region of at least `map_size * 8` bytes, and that no other code will access this memory for the duration of the bitmap's use.
    pub const unsafe fn new(map_address: VirtualAddress, map_size: usize) -> BitMap {
        BitMap {
            map: unsafe { slice::from_raw_parts_mut(map_address.as_mut_ptr::<u64>(), map_size) },
        }
    }

    /// Initializes the bitmap by setting all bits to zero.
    ///
    /// This method clears the entire bitmap, marking all bits as unset.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(addr, size);
    /// bitmap.init();
    /// assert_eq!(bitmap.count_ones(), 0);
    /// ```
    pub fn init(&mut self) {
        self.map.fill(0);
    }

    /// Sets all bits in the u64 at the specified position to 1.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 128);
    /// let pos = Position::new(0, 0).unwrap();
    /// bitmap.set_index(&pos);
    /// assert_eq!(bitmap[0], u64::MAX);
    /// ```
    pub fn set_index(&mut self, p: &Position) {
        self.map[p.map_index] = u64::MAX;
    }

    /// Sets a single bit in the bitmap at the specified position.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 128); // assume valid address and size
    /// let pos = Position::new(0, 3).unwrap();
    /// bitmap.set_bit(&pos);
    /// assert!(bitmap.get_bit(&pos));
    /// ```
    pub fn set_bit(&mut self, p: &Position) {
        self.map[p.map_index] |= 1 << p.bit_index;
    }

    /// Returns whether the bit at the specified position is set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 128); // assume valid address and size
    /// let pos = Position::new(0, 3).unwrap();
    /// bitmap.set_bit(&pos);
    /// assert!(bitmap.get_bit(&pos));
    /// ```
    pub fn get_bit(&self, p: &Position) -> bool {
        self.map[p.map_index] & (1 << p.bit_index) != 0
    }

    /// Sets a contiguous block of bits in the bitmap to 1, starting at the given position and using the provided block layout.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the range defined by `p` and `block` is valid within the bitmap's bounds. Out-of-bounds access may cause undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 128);
    /// let pos = Position::new(0, 3).unwrap();
    /// let block = ContiguousBlockLayout::from_start_size(&pos, 70);
    /// unsafe { bitmap.set_contiguous_block(&pos, &block); }
    /// ```
    pub unsafe fn set_contiguous_block(&mut self, p: &Position, block: &ContiguousBlockLayout) {
        self.map[p.map_index] |= block.low_mask;
        for i in 1..=block.index_count {
            self.map[p.map_index + i] = u64::MAX;
        }
        self.map[p.map_index + block.index_count + 1] |= block.high_mask;
    }

    /// Returns the total number of set bits (ones) in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 128);
    /// bitmap.init();
    /// assert_eq!(bitmap.count_ones(), 0);
    /// ```
    pub fn count_ones(&self) -> usize {
        self.map
            .iter()
            .map(|x| x.count_ones() as usize)
            .sum::<usize>()
    }

    /// Counts the total number of zero bits in the bitmap.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 2); // Assume 2 u64s, all bits zero
    /// bitmap.init();
    /// assert_eq!(bitmap.count_zeros(), 128);
    /// ```
    pub fn count_zeros(&self) -> usize {
        self.map
            .iter()
            .map(|x| x.count_zeros() as usize)
            .sum::<usize>()
    }

    /// Searches for a contiguous free block of bits of the specified size within the bitmap.
    ///
    /// Returns the starting position and block layout if a suitable free block is found, or `None` if no such block exists or if `bit_count` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut bitmap = BitMap::new(some_address, 128);
    /// bitmap.init();
    /// if let Some((pos, layout)) = bitmap.find_free_block(16) {
    ///     // pos is the start of a free 16-bit block
    /// }
    /// ```
    pub fn find_free_block(&self, bit_count: usize) -> Option<(Position, ContiguousBlockLayout)> {
        if bit_count == 0 {
            return None;
        }
        let (mut block, mut end_position) = ContiguousBlockLayout::initial_from_bits(bit_count);
        let mut start_position = unsafe { Position::new_unchecked(0, 0) };
        while end_position.map_index < self.map.len() {
            let window = &self.map[start_position.map_index..end_position.map_index];
            let (start, remain) = window
                .split_first()
                .expect("Could not extract first element");
            let (end, middle) = remain.split_last().expect("Could not extract last element");
            if middle.iter().sum::<u64>() == 0
                && start & block.low_mask == 0
                && end & block.high_mask == 0
            {
                return Some((start_position, block));
            }
            let val = block.shift();
            start_position.bit_index += 1;
            start_position.bit_index &= u64::BITS as usize - 1;
            start_position.map_index += (start_position.bit_index == 0) as usize;
            end_position.map_index += val as usize;
        }

        None
    }
}
