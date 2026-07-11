#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(explicit_tail_calls)]
#![feature(min_specialization)]
#![feature(ptr_alignment_type)]

pub mod meta;

use core::{alloc::GlobalAlloc, marker::PhantomData, ptr::NonNull};

use libk::println;
use sync::mutex::SpinMutex;
use x86::memory_map::MemoryMap;

use common::{
    address_types::Address, constants::REGULAR_PAGE_ALIGNMENT,
    enums::BuddyOrder, iter, volatile::Volatile,
};

use crate::meta::{BuddyArena, BuddyBlock, BuddyError, BuddyMeta, Head};

pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    arena: Arena,
    freelist: SpinMutex<[BuddyMeta<Head>; BuddyOrder::MAX as usize + 1]>,
    _block: PhantomData<Block>,
}

impl<Arena, Block> BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    pub fn new(memory_map: &MemoryMap) -> BuddyAllocator<Arena, Block> {
        let freelist = SpinMutex::new(
            [BuddyMeta::<Head>::default(); BuddyOrder::MAX as usize + 1],
        );

        let mut lock = freelist.lock();

        let head = &mut lock[0];

        let arena = Arena::new(memory_map, head);

        drop(lock);

        BuddyAllocator {
            arena,
            freelist,
            _block: PhantomData,
        }
    }

    pub fn initialize(&mut self) {
        let len = self.arena.iter().len();

        let mut first = self.arena.at(0).unwrap();

        let lock = self.freelist.lock();
        let head = &lock[0];
        unsafe {
            first.as_mut().meta_mut().prev =
                Volatile::new(NonNull::from_ref(head));
        }

        drop(lock);

        for (n, _power) in
            iter::power_chunk_firsts(0..len, BuddyOrder::MAX as usize)
        {
            self.merge_recursive(self.arena.at(n).unwrap());
        }

        todo!(
            "Allocate all the reserved spots in the memory map, and take \
             allocations from the bump allocator."
        )
    }

    // pub fn free_pages(&self, address: usize) {
    //     let page_index = address / REGULAR_PAGE_SIZE;
    // }

    /// This function assumes that `wanted_order` is empty, and won't check
    /// it.
    pub fn split_until(
        &self,
        wanted_order: usize,
    ) -> Option<NonNull<Block>> {
        let (closet_order, initial_page) = ((wanted_order + 1)
            ..=BuddyOrder::MAX as usize)
            .find_map(|i| {
                Some((
                    i,
                    Block::from_meta(self.freelist.lock()[i].next.read()?),
                ))
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
        &self,
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

        self.freelist.lock()[next_order].attach_block(rhs);

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
                Err(e) => {
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

    fn attach_block(&self, block: NonNull<Block>) {
        let order = unsafe { block.as_ref().meta().flags.get_order() };
        self.freelist.lock()[order as usize].attach_block(block);
    }
}

impl<Arena, Block> ::core::fmt::Debug for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BuddyAllocator")
            .field("freelist", &self.freelist.try_lock())
            .finish()
    }
}

unsafe impl<Arena, Block> GlobalAlloc for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let num_pages = layout.size().next_multiple_of(
            REGULAR_PAGE_ALIGNMENT.max(layout.alignment()).as_usize(),
        );

        debug_assert!(
            num_pages <= (1 << BuddyOrder::MAX as usize),
            "Size cannot be greater then: {}",
            1 << BuddyOrder::MAX as usize
        );
        if num_pages > (1 << BuddyOrder::MAX as usize) || num_pages == 0 {
            return core::ptr::null_mut();
        }

        let order = (usize::BITS
            - 1
            - num_pages.next_power_of_two().leading_zeros())
            as usize;

        let mut page =
            self.freelist.lock()[order].next.read().unwrap_or_else(|| {
                NonNull::from_ref(unsafe {
                    self.split_until(order)
                        .expect("Out of memory, swap is not implemented")
                        .as_ref()
                        .meta()
                })
            });

        // Detach page from the freelist and mark it as allocated
        unsafe {
            page.as_mut().detach();
            page.as_mut().flags.set_allocated(true);
        };

        self.arena
            .address_of(Block::from_meta(page))
            .as_non_null::<u8>()
            .as_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {}
}
