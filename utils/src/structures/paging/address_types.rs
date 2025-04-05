use constants::enums::PageSize;
use super::get_current_page_table;
use crate::impl_math_ops;

#[derive(Clone)]
pub struct PhysicalAddress(usize);

impl_math_ops!(PhysicalAddress, usize);

#[derive(Clone)]
pub struct VirtualAddress(usize);

impl_math_ops!(VirtualAddress, usize);

impl PhysicalAddress {
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn address(&self) -> usize {
        self.0
    }

    pub fn map(&self, address: VirtualAddress, page_size: PageSize) {
        address.map(self.clone(), page_size)
    }

}

impl VirtualAddress {
    pub const unsafe fn new_unchecked(address: usize) -> Self {
        Self(address)
    }
    pub const fn new(address: usize) -> Self {
        // Bitshift is done to make sure bits 63-48 are equal to bit 47
        Self((address << 16) >> 16)
    }

    // This function will obtain the page table automatically from cr3
    pub fn map(&self, address: PhysicalAddress, page_size: PageSize) {
        let mut table = unsafe { get_current_page_table() };

        for table_number in ((page_size.clone() as usize + 1)..=4).rev() {
            
            if table.entries[self.nth_pt_index(table_number)].present() {
                table = table.entries[self.nth_pt_index(table_number)].get_next_table_mut()
            } else {
                table.entries[self.nth_pt_index(table_number)].create_new_table();
            }
        }
        table.entries[self.nth_pt_index(page_size as usize)].set_frame_address(address);

    }

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
    // index of the n_th paage table
    pub const fn nth_pt_index(&self, n: usize) -> usize {
        if n > 4 || n < 1 {
            panic!("There are only 4 page tables, you tried to index table out of range");
        }
        (self.0 >> (39 - 9 * (4 - n))) & 0o777
    }


}