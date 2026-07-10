#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(explicit_tail_calls)]
#![feature(min_specialization)]

pub mod meta;

use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

use libk::println;
use x86::{memory_map::MemoryMap, structures::paging::PageTable};

use common::{
    address_types::{Address, PhysicalAddress},
    enums::BuddyOrder,
    iter,
    volatile::Volatile,
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
        let mut freelist =
            [BuddyMeta::<Head>::default(); BuddyOrder::MAX as usize + 1];

        let arena = Arena::new(memory_map, &mut freelist);

        BuddyAllocator {
            arena,
            freelist,
            _block: PhantomData,
        }
    }

    pub fn initialize(&mut self) {
        let len = self.arena.iter().len();

        let mut first = self.arena.at(0).unwrap();

        unsafe {
            first.as_mut().meta_mut().prev =
                Volatile::new(NonNull::from_ref(&self.freelist[0]));
        }

        for (n, _power) in
            iter::power_chunk_firsts(0..len, BuddyOrder::MAX as usize)
        {
            self.merge_recursive(self.arena.at(n).unwrap());
        }
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

        let page = self.freelist[order].next.read().unwrap_or_else(|| {
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
        let (closet_order, initial_page) = ((wanted_order + 1)
            ..=BuddyOrder::MAX as usize)
            .find_map(|i| {
                Some((i, Block::from_meta(self.freelist[i].next.read()?)))
            })?;

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
    pub fn merge_recursive(
        &mut self,
        mut page: NonNull<Block>,
    ) -> BuddyOrder {
        loop {
            let buddy = match self.arena.buddy_of(page) {
                Ok(buddy) => buddy,
                Err(
                    BuddyError::BuddyOutOfRange | BuddyError::MaxOrder,
                ) => {
                    let order =
                        unsafe { page.as_ref().meta().flags.get_order() };
                    return order;
                }
                Err(
                    e @ (BuddyError::PageInLargerOrder
                    | BuddyError::Unsplitable),
                ) => {
                    unreachable!(
                        "Problem in algorithm, the error should not \
                         happen {:?}\n {:?}",
                        e, page,
                    )
                }
            };

            if unsafe { buddy.as_ref().meta().flags.is_allocated() } {
                let order =
                    unsafe { page.as_ref().meta().flags.get_order() };
                return order;
            }

            let buddy_order =
                unsafe { buddy.as_ref().meta().flags.get_order() };
            let page_order =
                unsafe { page.as_ref().meta().flags.get_order() };

            debug_assert!(buddy_order <= page_order);

            if buddy_order < page_order {
                page = buddy;
                continue;
            }
            match self.arena.merge(page, buddy) {
                Ok(merged) => {
                    self.attach_block(merged);
                    page = merged;
                }
                Err(_e) => todo!("Handle Error: {:?}", _e),
            };
        }
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

    fn attach_block(&mut self, block: NonNull<Block>) {
        let order = unsafe { block.as_ref().meta().flags.get_order() };
        self.freelist[order as usize].attach_block(block);
    }
}

impl<Arena, Block> ::core::fmt::Debug for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BuddyAllocator")
            .field("freelist", &self.freelist)
            .finish()
    }
}
