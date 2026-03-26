#![no_std]
#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(explicit_tail_calls)]
pub mod meta;

use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

use x86::structures::paging::PageTable;

use common::{
    address_types::PhysicalAddress,
    enums::{BUDDY_MAX_ORDER, BuddyOrder},
};

use crate::meta::{BuddyArena, BuddyBlock, BuddyError, BuddyMeta, Head};

pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    arena: NonNull<Arena>,
    freelist: [BuddyMeta<Head>; BUDDY_MAX_ORDER],
    _block: PhantomData<Block>,
}

impl<Arena, Block> BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
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

        unsafe { self.arena.as_ref().address_of(Block::from_meta(page)) }
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
            ((wanted_order + 1)..BUDDY_MAX_ORDER).find_map(|i| {
                Some((i, Block::from_meta(self.freelist[i].next?)))
            })?;

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

        if current_order == target_order {
            return page;
        }

        let (lhs, rhs) = unsafe { self.arena.as_ref().split(page) };

        let next_order = current_order - 1;

        self.freelist[next_order].attach_block(rhs);

        become self.split_recursive(lhs, next_order, target_order)
    }

    /// This function will try to merge a page with it's buddy, until it
    /// cannot be merged anymore.
    pub fn merge_recursive(&mut self, page: NonNull<Block>) {
        let buddy = match unsafe { self.arena.as_ref().buddy_of(page) } {
            Ok(buddy) => buddy,
            Err(BuddyError::MaxOrder) => {
                self.freelist[BUDDY_MAX_ORDER - 1].attach_block(page);
                return;
            }
        };

        if unsafe { buddy.as_ref().meta().flags.is_allocated() } {
            self.freelist[unsafe {
                page.as_ref().meta().flags.get_order() as usize
            }]
            .attach_block(page);
            return;
        }

        let (mut left, mut right) = (page, buddy);
        unsafe {
            if self.arena.as_ref().address_of(left)
                > self.arena.as_ref().address_of(right)
            {
                core::mem::swap(&mut left, &mut right);
            }
        }

        let merged = unsafe { self.arena.as_mut().merge(left, right) };

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
