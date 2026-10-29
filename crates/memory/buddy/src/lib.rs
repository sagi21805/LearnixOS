#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(min_specialization)]
#![feature(ptr_alignment_type)]

pub mod meta;

use core::{
    alloc::{AllocError, Allocator, Layout},
    cmp::Ordering,
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

use crate::meta::{BuddyArena, BuddyBlock, BuddyMeta, Head};

pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    inner: SpinMutex<BuddyAllocatorInner<Arena, Block>>,
}

impl<Arena, Block> BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    pub fn new(memory_map: &MemoryMap) -> BuddyAllocator<Arena, Block> {
        let mut freelist =
            [BuddyMeta::<Head>::default(); BuddyOrder::MAX as usize + 1];
        let arena = Arena::new(memory_map, &mut freelist[0]);

        BuddyAllocator {
            inner: SpinMutex::new(BuddyAllocatorInner {
                arena,
                freelist,
                _block: PhantomData,
            }),
        }
    }

    pub fn initialize(
        &self,
        allocations: &BumpAllocations,
        mmap: &MemoryMap,
    ) {
        let mut inner = self.inner.lock();
        let len = inner.arena.iter().len();

        let mut first = inner.arena.at(0).unwrap();
        unsafe {
            first.as_mut().meta_mut().prev =
                Some(NonNull::from_mut(&mut inner.freelist[0].next));
        }

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
                let mut block = inner
                    .arena
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
                        block.as_mut().meta_mut().detach();
                        inner.mark_allocated(block);
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

        let mut block =
            inner.arena.page_with_address(allocation_address).unwrap();

        println!("Block: {:?}", block);

        let mut length =
            allocation.layout.size().next_multiple_of(REGULAR_PAGE_SIZE);

        while length > 0 {
            length -= REGULAR_PAGE_SIZE;
            unsafe {
                block.as_mut().meta_mut().detach();
                inner.mark_allocated(block);
            }
            block = unsafe { block.add(1) };
        }

        for (n, _power) in
            iter::power_chunk_firsts(0..len, BuddyOrder::MAX as usize)
        {
            let page = inner.arena.at(n).unwrap();
            inner.merge(page);
        }
    }

    pub fn print_allocated_regions(&self) {
        let inner = self.inner.lock();

        let mut prev_address =
            unsafe { PhysicalAddress::new_unchecked(0) };

        for block in inner.arena.iter() {
            let meta = unsafe { block.as_ref().meta() };

            if meta.flags.is_allocated() {
                let address = inner.arena.address_of(block);
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
    Block: BuddyBlock,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let inner = self.inner.try_lock();
        f.debug_struct("BuddyAllocator")
            .field(
                "freelist",
                &inner.as_ref().map(|inner| &inner.freelist),
            )
            .finish()
    }
}

// SAFETY: every block comes from this allocator's arena and free lists,
// stays allocated until `deallocate` returns it, and is never handed out
// twice.
unsafe impl<Arena, Block> Allocator for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    /// Takes a block of the smallest order that fits `layout`, and
    /// returns all of it.
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let order = order_of(layout).ok_or(AllocError)?;

        let mut inner = self.inner.lock();
        let block = inner
            .pop(order)
            .or_else(|| inner.split_until(order))
            .ok_or(AllocError)?;
        // SAFETY: `pop` and `split_until` return a free block of the
        // arena, which they detached.
        unsafe { inner.mark_allocated(block) };

        let start = inner.arena.address_of(block).as_non_null::<u8>();
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

        let mut inner = self.inner.lock();
        let mut page = inner
            .arena
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
        inner.freelist[order].attach_block(page);
    }
}

/// An arena and the free lists that link its blocks.
struct BuddyAllocatorInner<Arena, Block> {
    arena: Arena,
    freelist: [BuddyMeta<Head>; BuddyOrder::MAX as usize + 1],
    _block: PhantomData<Block>,
}

impl<Arena, Block> BuddyAllocatorInner<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    /// Splits the smallest free block above `order` down to `order`, and
    /// returns its first half, detached. Each split puts the second half
    /// in the free list one order down.
    fn split_until(&mut self, order: usize) -> Option<NonNull<Block>> {
        let (mut current, mut block) = (order + 1
            ..=BuddyOrder::MAX as usize)
            .find_map(|i| Some((i, self.pop(i)?)))?;

        while current > order {
            let (lhs, rhs) = self
                .arena
                .split(block)
                .expect("a block above `order` splits into two");
            current -= 1;
            self.attach(rhs);
            block = lhs;
        }
        Some(block)
    }

    /// Merges `block` with its buddy, and the result with its own buddy,
    /// until a buddy is allocated or missing. A buddy of a lower order is
    /// merged up first.
    fn merge(&mut self, mut block: NonNull<Block>) {
        while let Some(buddy) = self.arena.free_buddy_of(block) {
            // SAFETY: `block` and its buddy are live blocks of the arena.
            let (order, buddy_order) = unsafe {
                (block.as_ref().order(), buddy.as_ref().order())
            };
            block = match buddy_order.cmp(&order) {
                Ordering::Less => buddy,
                Ordering::Equal => {
                    let merged = self
                        .arena
                        .merge(block, buddy)
                        .expect("free buddies of one order merge");
                    self.attach(merged);
                    merged
                }
                Ordering::Greater => {
                    unreachable!("a buddy is never larger than its block")
                }
            };
        }
    }

    /// Detaches the first block of the free list of `order`.
    fn pop(&mut self, order: usize) -> Option<NonNull<Block>> {
        let mut block =
            Block::from_meta(self.freelist[order].next?.cast());
        // SAFETY: free lists link only live blocks of the arena.
        unsafe { block.as_mut() }.meta_mut().detach();
        Some(block)
    }

    /// Links `block` into the free list of its order.
    fn attach(&mut self, block: NonNull<Block>) {
        // SAFETY: the allocator attaches only live blocks of its arena.
        let order = unsafe { block.as_ref() }.order();
        self.freelist[order as usize].attach_block(block);
    }

    /// Marks a detached block as allocated.
    ///
    /// # Safety
    ///
    /// `block` must be a free block of the arena, in no free list.
    unsafe fn mark_allocated(&mut self, mut block: NonNull<Block>) {
        // SAFETY: the caller guarantees `block` is a live block of the
        // arena.
        let meta = unsafe { block.as_mut() }.meta_mut();
        debug_assert!(
            !meta.flags.is_allocated() && meta.prev.is_none(),
            "{block:?} is free and in no free list"
        );
        meta.flags.set_allocated(true);
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
