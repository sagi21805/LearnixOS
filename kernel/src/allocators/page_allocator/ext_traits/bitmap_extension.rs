use crate::allocators::bitmap::BitMap;
use crate::println;
use constants::enums::PageSize;

pub(in super::super) trait BitMapExtension {
    unsafe fn set_page_unchecked(&mut self, map_index: usize, bit_index: u32, page_size: PageSize);

    fn print_bits(&self, map_index: usize);
}

#[allow(unsafe_op_in_unsafe_fn)]
impl BitMapExtension for BitMap {
    unsafe fn set_page_unchecked(&mut self, map_index: usize, bit_index: u32, page_size: PageSize) {
        match page_size {
            PageSize::Regular => {
                self.set_bit_unchecked(map_index, bit_index);
            }

            PageSize::Big | PageSize::Huge => {
                for index in
                    map_index..(map_index + (page_size.size_in_pages() / u64::BITS as usize))
                {
                    self.set_index_unchecked(index);
                }
            }
        }
    }

    fn print_bits(&self, map_index: usize) {
        let value = self.map[map_index];
        println!("Entry: {:064b}", value);
    }
}
