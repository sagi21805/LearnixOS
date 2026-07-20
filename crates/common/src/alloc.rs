use core::{alloc::Layout, mem::Alignment};

use crate::constants::BUMP_TOTAL_ALLOCATIONS;

use crate::address_types::{Address, VirtualAddress};

#[derive(Copy, Clone, Debug)]
pub struct Allocation {
    pub layout: core::alloc::Layout,
    pub base: VirtualAddress,
}

#[rustfmt::skip]
impl const Default for Allocation {
    fn default() -> Self {
        Allocation {
            layout: unsafe {
                Layout::from_size_alignment_unchecked(
                    0,
                    Alignment::new_unchecked(1),
                )
            },
            base: VirtualAddress::null(),
        }
    }
}

impl Allocation {
    pub fn is_null(&self) -> bool { self.base.as_usize() == 0 }
}

pub struct Allocations<const N: usize> {
    allocations: [Allocation; N],
    pub index: usize,
}

#[rustfmt::skip]
impl<const N: usize> const Default for Allocations<N> {
    fn default() -> Self {
        Allocations {
            allocations: [Allocation::default(); N],
            index: 0,
        }
    }
}

impl<const N: usize> Allocations<N> {
    pub fn write(&mut self, a: Allocation) {
        self.allocations[self.index] = a;
        self.index += 1;
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Allocation> + '_ {
        self.allocations.iter().take(self.index)
    }
}

pub type BumpAllocations = Allocations<BUMP_TOTAL_ALLOCATIONS>;
