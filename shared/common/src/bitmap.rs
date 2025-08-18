use crate::address_types::VirtualAddress;
use core::{slice, u64};
use derive_more::Constructor;

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub map_index: usize,
    pub bit_index: usize,
}
impl Position {
    /// Create a new position for the map and bit index.
    ///
    /// This function will return None if the `bit_index` >= 64
    /// because its indices exceeds array with length  [`u64::BITS`]  
    pub fn new(map_index: usize, bit_index: usize) -> Option<Self> {
        if bit_index < 64 {
            Some(Position {
                map_index,
                bit_index,
            });
        }
        None
    }

    /// Create a position for the map and bit index without checking if `bit_index` < 64
    pub const unsafe fn new_unchecked(map_index: usize, bit_index: usize) -> Position {
        Position {
            map_index,
            bit_index,
        }
    }

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

    /// Create a new contiguous block from a size and a start position.
    ///
    /// # Parameters
    ///
    /// - `p`: The start position of the block.
    /// - `size`: The size of the block in bits.
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

    /// Resets the block to it's initial value without checking self.low_mask value
    ///
    /// # Safety
    ///
    /// This method expects that self.low_mask is 0!
    fn reset_unchecked(&mut self) {
        let is_zero = self.index_count == 0;
        self.index_count -= is_zero as usize;
        self.low_mask = self.high_mask | (u64::MAX + !is_zero as u64);
        self.high_mask &= u64::MAX + !is_zero as u64;
    }

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

    fn index(&self, index: usize) -> &Self::Output {
        &self.map[index]
    }
}

impl core::ops::IndexMut<usize> for BitMap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.map[index]
    }
}

impl BitMap {
    /// Creates a new bitmap structure taking ownership on the map_address up to map_address + size
    ///
    /// All of the indexes will start with a default value of zero
    ///
    /// # Parameters
    ///
    /// - `map_address`: That address of the map which this structure will assume is owned by himself
    /// - `map_size`: The number of indexes the map array will have.
    ///               This will result in `size * 64` one bit indexes in the map itself
    ///
    /// # Safety
    ///
    /// The virtual address that is given to this structure is assumed to be owned by this structure.
    pub const unsafe fn new(map_address: VirtualAddress, map_size: usize) -> BitMap {
        BitMap {
            map: unsafe { slice::from_raw_parts_mut(map_address.as_mut_ptr::<u64>(), map_size) },
        }
    }

    /// Initlize the bitmap by filling all of it in zeros
    pub fn init(&mut self) {
        self.map.fill(0);
    }

    /// Set all of bit of the u64 into 1
    pub fn set_index(&mut self, p: &Position) {
        self.map[p.map_index] = u64::MAX;
    }

    pub fn unset_index(&mut self, p: &Position) {
        self.map[p.map_index] = 0;
    }

    pub fn unset_bit(&mut self, p: &Position) {
        self.map[p.map_index] &= !(1 << p.bit_index)
    }

    /// Sets the bit corresponding to the `map_index` and the `bit_index`
    pub fn set_bit(&mut self, p: &Position) {
        self.map[p.map_index] |= 1 << p.bit_index;
    }

    /// Gets the bit corresponding to the given position
    pub fn get_bit(&self, p: &Position) -> bool {
        self.map[p.map_index] & (1 << p.bit_index) != 0
    }

    /// Set a contiguous block, the contiguous low mask, and high mask should be relative to the positions map index.
    pub unsafe fn set_contiguous_block(&mut self, p: &Position, block: &ContiguousBlockLayout) {
        if (p.map_index + block.index_count + 1) > self.map.len() {
            panic!("The block passes the bound of this map!")
        }
        self.map[p.map_index] |= block.low_mask;
        for i in 1..=block.index_count {
            self.map[p.map_index + i] = u64::MAX;
        }
        self.map[p.map_index + block.index_count + 1] |= block.high_mask;
    }

    pub fn count_ones(&self) -> usize {
        self.map
            .iter()
            .map(|x| x.count_ones() as usize)
            .sum::<usize>()
    }

    pub fn count_zeros(&self) -> usize {
        self.map
            .iter()
            .map(|x| x.count_zeros() as usize)
            .sum::<usize>()
    }

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
