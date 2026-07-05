#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(explicit_tail_calls)]

#[cfg(feature = "host")]
extern crate std;

#[cfg(not(feature = "host"))]
use libk::println;
#[cfg(feature = "host")]
use std::println;

pub mod meta;

use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

use x86::{memory_map::MemoryMap, structures::paging::PageTable};

use common::{
    address_types::{Address, PhysicalAddress},
    enums::BuddyOrder,
    iter,
};

use crate::meta::{BuddyArena, BuddyBlock, BuddyError, BuddyMeta, Head};

pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    arena: Arena,
    freelist: [BuddyMeta<Head>; BuddyOrder::MAX as usize + 1],
    _block: PhantomData<Block>,
}

impl<Arena, Block> BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    pub fn new(memory_map: &MemoryMap) -> BuddyAllocator<Arena, Block> {
        let freelist =
            [BuddyMeta::<Head>::default(); BuddyOrder::MAX as usize + 1];

        let arena = Arena::new(memory_map, &freelist);

        let mut allocator = BuddyAllocator {
            arena,
            freelist,
            _block: PhantomData,
        };

        let len = allocator.arena.iter().len();

        for (n, _power) in
            iter::power_chunk_firsts(0..len, BuddyOrder::MAX as usize)
        {
            allocator.merge_recursive(allocator.arena.at(n).unwrap());
        }

        allocator
    }

    pub fn alloc_pages(&mut self, num_pages: usize) -> PhysicalAddress {
        assert!(
            num_pages <= (1 << BuddyOrder::MAX as usize),
            "Size cannot be greater then: {}",
            1 << BuddyOrder::MAX as usize
        );
        let order = (usize::BITS
            - 1
            - num_pages.next_power_of_two().leading_zeros())
            as usize;

        let page = self.freelist[order].next.unwrap_or_else(|| {
            NonNull::from_ref(unsafe {
                self.split_until(order)
                    .expect("Out of memory, swap is not implemented")
                    .as_ref()
                    .meta()
            })
        });

        self.arena.address_of(Block::from_meta(page))
    }

    // pub fn free_pages(&self, address: usize) {
    //     let page_index = address / REGULAR_PAGE_SIZE;
    // }

    /// This function assumes that `wanted_order` is empty, and won't check
    /// it.
    pub fn split_until(
        &mut self,
        wanted_order: usize,
    ) -> Option<NonNull<Block>> {
        let (closet_order, initial_page) =
            ((wanted_order + 1)..=BuddyOrder::MAX as usize).find_map(
                |i| Some((i, Block::from_meta(self.freelist[i].next?))),
            )?;

        println!(
            "Initial Page: {:?}, Closet Order: {}, Wanted Order: {}",
            initial_page, closet_order, wanted_order
        );

        Some(self.split_recursive(
            initial_page,
            closet_order,
            wanted_order,
        ))
    }

    fn split_recursive(
        &mut self,
        page: NonNull<Block>,
        current_order: usize,
        target_order: usize,
    ) -> NonNull<Block> {
        debug_assert!(
            target_order < current_order,
            "Target order cannot be greater then current order"
        );

        println!(
            "Target Order: {}, Current Order: {}",
            target_order, current_order
        );

        if current_order == target_order {
            return page;
        }

        let (lhs, rhs) = match self.arena.split(page) {
            Ok((lhs, rhs)) => (lhs, rhs),
            Err(e) => todo!("Handle Error, {:?}", e),
        };

        let next_order = current_order - 1;

        self.freelist[next_order].attach_block(rhs);

        become self.split_recursive(lhs, next_order, target_order)
    }

    /// This function will try to merge a page with it's buddy, until it
    /// cannot be merged anymore.
    pub fn merge_recursive(&mut self, page: NonNull<Block>) {
        let buddy = match self.arena.buddy_of(page) {
            Ok(buddy) => buddy,
            Err(BuddyError::MaxOrder) => {
                self.freelist[BuddyOrder::MAX as usize].attach_block(page);
                return;
            }
            Err(BuddyError::BuddyOutOfRange) => {
                self.freelist[unsafe {
                    page.as_ref().meta().flags.get_order() as usize
                }]
                .attach_block(page);
                return;
            }
            Err(
                BuddyError::PageInLargerOrder | BuddyError::Unsplitable,
            ) => unreachable!(
                "Problem in algorithm, the error should not happen"
            ),
        };

        if unsafe { buddy.as_ref().meta().flags.is_allocated() } {
            // Attach block, cannot be merged anymore.
            self.freelist[unsafe {
                page.as_ref().meta().flags.get_order() as usize
            }]
            .attach_block(page);
            return;
        }

        let merged = match self.arena.merge(page, buddy) {
            Ok(merged) => merged,
            Err(_e) => todo!("Handle Error: {:?}", _e),
        };

        become BuddyAllocator::merge_recursive(self, merged);
    }

    pub fn alloc_table(&mut self) -> NonNull<PageTable> {
        unsafe {
            let address = self.alloc_pages(1).translate();
            ptr::write_volatile(
                address.as_non_null::<PageTable>().as_ptr(),
                PageTable::empty(),
            );
            address.as_non_null::<PageTable>()
        }
    }
}
