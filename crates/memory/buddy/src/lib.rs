#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(explicit_tail_calls)]
#![feature(min_specialization)]
#![feature(ptr_alignment_type)]

pub mod meta;

use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use libk::println;
use sync::mutex::SpinMutex;
use x86::{memory_map::MemoryMap, structures::paging::VirtualAddressExt};

use common::{
    address_types::{Address, PhysicalAddress},
    alloc::BumpAllocations,
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    enums::{BuddyOrder, MemoryRegionType},
    iter,
};

use crate::meta::{
    BuddyArena, BuddyBlock, BuddyError, BuddyMeta, Head, Regular,
};

pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    arena: SpinMutex<Arena>,
    freelist: SpinMutex<[BuddyMeta<Head>; BuddyOrder::MAX as usize + 1]>,
    // Wrap in a mutex to automatically implement Sync and Send.
    _block: PhantomData<SpinMutex<Block>>,
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

        let arena = SpinMutex::new(Arena::new(memory_map, head));

        drop(lock);

        BuddyAllocator {
            arena,
            freelist,
            _block: PhantomData,
        }
    }

    pub fn initialize(
        &self,
        allocations: &BumpAllocations,
        mmap: &MemoryMap,
    ) {
        let len = self.arena.lock().iter().len();

        let mut first = self.arena.lock().at(0).unwrap();

        let mut lock = self.freelist.lock();
        let head = &mut lock[0];
        unsafe {
            first.as_mut().meta_mut().prev =
                Some(NonNull::from_mut(&mut head.next));
        }

        drop(lock);

        // Allocate all the non-usable regions in the memory map
        if let Some(last_usable_idx) = mmap
            .regions
            .read()
            .iter()
            .rposition(|r| r.region_type == MemoryRegionType::Usable)
        {
            for region in mmap
                .regions
                .read()
                .iter()
                .take(last_usable_idx + 1)
                .filter(|r| r.region_type != MemoryRegionType::Usable)
            {
                let mut block = self
                    .arena
                    .lock()
                    .page_with_address(unsafe {
                        PhysicalAddress::new_unchecked(
                            region.base_address as usize,
                        )
                        .align_down(REGULAR_PAGE_ALIGNMENT)
                    })
                    .unwrap();
                println!("First Block: {:?}", block);

                let mut length = (region.length as usize)
                    .next_multiple_of(REGULAR_PAGE_SIZE);

                while length > 0 {
                    length -= REGULAR_PAGE_SIZE;
                    unsafe {
                        self.allocate_block(block);
                    }
                    block = unsafe { block.add(1) };
                }
                println!(
                    "Allocated Range {:x}..{:x}",
                    PhysicalAddress::new_unchecked(
                        region.base_address as usize,
                    )
                    .align_down(REGULAR_PAGE_ALIGNMENT)
                    .as_usize(),
                    region.base_address + region.length
                )
            }
        }

        // Note: Because this is a hobby OS, I do not hanlde all cases on
        // the ranges in the memory map. And I rely on the observation that
        // the first first matching free range in the arena is a power of
        // 2, which is good for this allocator.
        if allocations.index > 1 {
            todo!(
                "Currently does not implement passing allocations that \
                 are not the arena of this allocator"
            )
        }

        let allocation = allocations.iter().nth(0).unwrap();

        println!("Allocation Base: {:x?}", allocation.base.as_usize());

        let allocation_address = allocation.base.translate().unwrap();

        println!(
            "Initial Allocation Address: {}",
            allocation_address.as_usize()
        );

        let mut block = self
            .arena
            .lock()
            .page_with_address(allocation_address)
            .unwrap();

        println!("Block: {:?}", block);

        let mut length =
            allocation.layout.size().next_multiple_of(REGULAR_PAGE_SIZE);

        while length > 0 {
            length -= REGULAR_PAGE_SIZE;
            unsafe {
                self.allocate_block(block);
            }
            block = unsafe { block.add(1) };
        }

        for (n, _power) in
            iter::power_chunk_firsts(0..len, BuddyOrder::MAX as usize)
        {
            let page = self.arena.lock().at(n).unwrap();
            self.merge_recursive(page);
        }
    }

    /// Mark a block as allocated, detaching it from the freelist.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `block` is a valid free block in the
    /// arena.
    unsafe fn allocate_meta(
        &self,
        mut block: NonNull<BuddyMeta<Regular>>,
    ) {
        debug_assert!(
            unsafe { !block.as_ref().flags.is_allocated() },
            "{:?}",
            block
        );

        // Detach page from the freelist and mark it as allocated
        unsafe {
            block.as_mut().detach();
            block.as_mut().flags.set_allocated(true);
        };
    }

    unsafe fn allocate_block(&self, block: NonNull<Block>) {
        unsafe {
            self.allocate_meta(NonNull::from_ref(block.as_ref().meta()));
        }
    }

    pub fn split_until(
        &self,
        wanted_order: usize,
    ) -> Option<NonNull<Block>> {
        let (closet_order, initial_page) = ((wanted_order + 1)
            ..=BuddyOrder::MAX as usize)
            .find_map(|i| {
                Some((
                    i,
                    Block::from_meta(self.freelist.lock()[i].next?.cast()),
                ))
            })?;

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
        debug_assert!(target_order < current_order);

        if current_order == target_order {
            return page;
        }

        let (lhs, rhs) = match self.arena.lock().split(page) {
            Ok((lhs, rhs)) => (lhs, rhs),
            Err(e) => todo!("Handle Error, {:?}", e),
        };

        let next_order = current_order - 1;

        self.freelist.lock()[next_order].attach_block(rhs);

        become self.split_recursive(lhs, next_order, target_order)
    }

    /// This function will try to merge a page with it's buddy, until it
    /// cannot be merged anymore.
    pub fn merge_recursive(&self, mut page: NonNull<Block>) -> BuddyOrder {
        let arena = self.arena.lock();
        loop {
            let buddy = match arena.buddy_of(page) {
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
            match arena.merge(page, buddy) {
                Ok(merged) => {
                    self.attach_block(merged);
                    page = merged;
                }
                Err(e) => unreachable!("{}", e),
            };
        }
    }

    fn attach_block(&self, block: NonNull<Block>) {
        let order = unsafe { block.as_ref().meta().flags.get_order() };
        self.freelist.lock()[order as usize].attach_block(block);
    }

    pub fn print_allocated_regions(&self) {
        let arena = self.arena.lock();

        let mut prev_address =
            unsafe { PhysicalAddress::new_unchecked(0) };

        for block in arena.iter() {
            let meta = unsafe { block.as_ref().meta() };

            if meta.flags.is_allocated() {
                let address = arena.address_of(block);
                if address.as_usize() - prev_address.as_usize()
                    > REGULAR_PAGE_SIZE
                {
                    println!("");
                }
                println!("{:x?}", address);
                prev_address = address;
            }
        }
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

/// The buddy order whose blocks fit `layout`, or `None` for an empty
/// layout or one larger than the biggest block.
fn order_of(layout: Layout) -> Option<usize> {
    let pages = layout.size().next_multiple_of(
        REGULAR_PAGE_ALIGNMENT.max(layout.alignment()).as_usize(),
    ) / REGULAR_PAGE_SIZE;

    (1..=1 << BuddyOrder::MAX as usize)
        .contains(&pages)
        .then(|| pages.next_power_of_two().ilog2() as usize)
}

// SAFETY: every block comes from this allocator's arena and free lists,
// stays allocated until `deallocate` returns it, and is never handed out
// twice.
unsafe impl<Arena, Block> Allocator for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: const BuddyBlock,
{
    /// Takes a block of the smallest order that fits `layout`, and
    /// returns all of it.
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let order = order_of(layout).ok_or(AllocError)?;

        let free = self.freelist.lock()[order].next.map(NonNull::cast);
        let page = match free {
            Some(page) => page,
            None => {
                let block = self.split_until(order).ok_or(AllocError)?;
                // SAFETY: `split_until` returns a live block of the arena.
                NonNull::from_ref(unsafe { block.as_ref() }.meta())
            }
        };

        // SAFETY: `page` heads the free list of `order`, or was just split
        // off for this call, so it is a free block.
        unsafe { self.allocate_meta(page) };

        let start = self
            .arena
            .lock()
            .address_of(Block::from_meta(page))
            .as_non_null::<u8>();
        Ok(NonNull::slice_from_raw_parts(
            start,
            REGULAR_PAGE_SIZE << order,
        ))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let order = order_of(layout);
        debug_assert!(order.is_some(), "`layout` came from `allocate`");
        // SAFETY: the caller guarantees `layout` is the one `allocate`
        // accepted, so it has an order.
        let order = unsafe { order.unwrap_unchecked() };

        let mut page = self
            .arena
            .lock()
            .page_with_address(PhysicalAddress::from(ptr.addr().get()))
            .expect("an allocated block lies in the arena");

        // SAFETY: `page` is the allocated block at `ptr`, which nothing
        // else uses once it is freed.
        unsafe {
            page.as_mut()
                .meta_mut()
                .flags
                .set_order(BuddyOrder::from(order as u8));
        }

        // Deallocate the page by attaching it back to the freelist.
        self.freelist.lock()[order].attach_block(page);
    }
}
