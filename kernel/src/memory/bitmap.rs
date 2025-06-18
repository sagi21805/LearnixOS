use core::{panic, slice, u64};
use cpu_utils::structures::paging::address_types::PhysicalAddress;
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
    /// because its inedex exceeds array with length  [`u64::BITS`]  
    pub fn new(map_index: usize, bit_index: usize) -> Option<Self> {
        if bit_index < 64 {
            Some(Position {
                map_index,
                bit_index,
            });
        }
        None
    }

    pub unsafe fn new_unchecked(map_index: usize, bit_index: usize) -> Position {
        Position {
            map_index,
            bit_index,
        }
    }
}

#[derive(Clone, Default, Constructor)]
pub struct ContiguousBlockLayout {
    pub low_mask: u64,
    pub index_count: usize,
    pub high_mask: u64,
}

impl ContiguousBlockLayout {
    pub fn initial_from_bits(bit_count: usize) -> (Self, Position) {
        let remaining_bits = bit_count.saturating_sub(u64::BITS as usize);
        let low_bit_count = bit_count - remaining_bits; // This nubmer must be less then or equal to 64
        let high_bit_count = remaining_bits % u64::BITS as usize;
        let index_count = remaining_bits / u64::BITS as usize;
        (
            Self::new(
                u64::MAX >> (u64::BITS - low_bit_count as u32),
                index_count,
                u64::MAX >> (u64::BITS - high_bit_count as u32),
            ),
            unsafe {
                Position::new_unchecked(
                    index_count + (low_bit_count / u64::MAX as usize),
                    high_bit_count,
                )
            },
        )
    }

    pub(self) fn shift(&mut self) -> u64 {
        self.low_mask <<= 1;
        self.high_mask <<= 1;
        self.high_mask += 1;
        let val = self.high_mask.saturating_sub(u64::MAX - 1) as u64;
        self.high_mask += val;
        self.index_count += val as usize;
        val
    }
}

/// A low-level bitmap structure
#[derive(Debug)]
pub struct BitMap {
    pub map: &'static mut [u64],
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
    pub const unsafe fn new(map_address: PhysicalAddress, map_size: usize) -> BitMap {
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

    /// Sets the bit corresponding to the `map_index` and the `bit_index`
    pub fn set_bit(&mut self, p: &Position) {
        self.map[p.map_index] |= 1 << p.bit_index;
    }

    /// Gets the bit corresponding to the given position
    pub fn get_bit(&self, p: &Position) -> bool {
        self.map[p.map_index] & (1 << p.bit_index) != 0
    }

    pub fn count_ones(&self) -> usize {
        self.map
            .iter()
            .map(|&x| x.count_ones() as usize)
            .sum::<usize>()
    }

    pub fn count_zeros(&self) -> usize {
        self.map
            .iter()
            .map(|&x| x.count_zeros() as usize)
            .sum::<usize>()
    }

    pub fn find_free_block(&self, bit_count: usize) -> Option<Position> {
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
                return Some(start_position);
            }
            // TODO think of a better name.
            let val = block.shift();
            start_position.bit_index += 1;
            if start_position.bit_index == 64 {
                start_position.bit_index = 0;
            }
            end_position.map_index += val as usize;
        }

        None
    }
}
