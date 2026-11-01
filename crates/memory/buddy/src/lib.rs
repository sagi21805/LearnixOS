#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(min_specialization)]
#![feature(ptr_alignment_type)]

mod init;
pub mod meta;
mod owned_block;

pub use owned_block::OwnedBlock;

use core::{
    alloc::{AllocError, Allocator, Layout},
    cmp::Ordering,
    marker::PhantomData,
    ptr::NonNull,
};

use sync::{
    mutex::{MutexGuard, SpinMutex},
    strategy::Spin,
};
use x86::memory_map::MemoryMap;

use common::{
    address_types::{Address, VirtualAddress},
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    enums::BuddyOrder,
};

use crate::meta::{
    Attached, BlockFlagBits, BuddyArena, BuddyBlock, BuddyError, Detached,
    FreeList, Link, Unlink,
};

/// The free lists of each order, `freelist[order]`.
type FreeLists = [FreeList; BuddyOrder::MAX as usize + 1];

/// A buddy allocator over the blocks of `Arena`.
///
/// The arena sits outside the lock, so anyone may read a block's flags
/// or find the block of an address without locking. Only the free lists,
/// and with them the right to hand blocks out and take them back, sit
/// behind it.
pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    arena: Arena,
    freelist: SpinMutex<FreeLists>,
    _block: PhantomData<Block>,
}

impl<Arena, Block> BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    pub fn new(memory_map: &MemoryMap) -> BuddyAllocator<Arena, Block> {
        BuddyAllocator {
            arena: Arena::new(memory_map),
            freelist: SpinMutex::new(
                [const { FreeList::empty() };
                    BuddyOrder::MAX as usize + 1],
            ),
            _block: PhantomData,
        }
    }

    /// Builds the free lists from the arena.
    ///
    /// Reserves the memory that isn't free, links every free page into the
    /// order-0 list, and merges buddies from there.
    pub fn initialize(&mut self, mmap: &MemoryMap) {
        self.locked().initialize(mmap);
    }

    /// The arena whose blocks this allocator hands out.
    pub fn arena(&self) -> &Arena { &self.arena }

    /// The block of the page at `ptr`, a pointer into memory that
    /// `allocate` handed out.
    pub fn block_of(
        &self,
        ptr: NonNull<u8>,
    ) -> Result<NonNull<Block>, BuddyError> {
        self.arena.page_with_virtual_address(VirtualAddress::from(
            ptr.addr().get(),
        ))
    }

    /// Takes a block of the smallest order that fits `layout`, and returns
    /// the right to write it.
    pub fn allocate_block(
        &self,
        layout: Layout,
    ) -> Result<OwnedBlock<Block>, AllocError> {
        let order = order_of(layout).ok_or(AllocError)?;
        self.locked().allocate_block(order).ok_or(AllocError)
    }

    /// Puts `block` back in its free list, and merges it with its free
    /// buddies.
    pub fn deallocate_block(&self, block: OwnedBlock<Block>) {
        self.locked().deallocate_block(block);
    }

    /// The arena and the locked free lists, until the result is dropped.
    fn locked(&self) -> BuddyAllocatorInner<'_, Arena, Block> {
        BuddyAllocatorInner {
            arena: &self.arena,
            freelist: self.freelist.lock(),
            _block: PhantomData,
        }
    }
}

impl<Arena, Block> ::core::fmt::Debug for BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let freelist = self.freelist.try_lock();
        f.debug_struct("BuddyAllocator")
            .field("freelist", &freelist.as_deref())
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
    /// returns all of it, through the higher-half map of physical memory.
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let block = self.allocate_block(layout)?.into_raw();
        // SAFETY: `allocate_block` returns a live block of the arena.
        let order = unsafe { block.as_ref() }.flags().order();
        let start =
            self.arena.address_of(block).translate().as_non_null::<u8>();
        Ok(NonNull::slice_from_raw_parts(
            start,
            REGULAR_PAGE_SIZE << order as usize,
        ))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let block = self
            .block_of(ptr)
            .expect("an allocated block lies in the arena");
        // SAFETY: `ptr` came from `allocate`, which gave the block's token
        // up.
        let block = unsafe { OwnedBlock::from_raw(block) };
        debug_assert_eq!(
            order_of(layout),
            Some(unsafe { block.as_ptr().as_ref() }.flags().order()
                as usize),
            "`layout` came from `allocate`"
        );
        self.deallocate_block(block);
    }
}

/// The arena and the locked free lists, for one operation.
pub(crate) struct BuddyAllocatorInner<'a, Arena, Block> {
    arena: &'a Arena,
    freelist: MutexGuard<'a, FreeLists, Spin>,
    _block: PhantomData<Block>,
}

impl<Arena, Block> BuddyAllocatorInner<'_, Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    /// Takes a block of `order`, and returns the right to write it, or
    /// `None` when no free block is large enough.
    fn allocate_block(
        &mut self,
        order: usize,
    ) -> Option<OwnedBlock<Block>> {
        let block = self.pop(order).or_else(|| self.split_until(order))?;
        // SAFETY: `pop` and `split_until` return a free block of the
        // arena, which they detached.
        unsafe { self.mark_allocated(block) };
        // SAFETY: the block was free until now, so nothing else owns it.
        Some(unsafe { OwnedBlock::from_raw(block) })
    }

    /// Puts `block` back in the free list of its order, and merges it with
    /// its free buddies.
    fn deallocate_block(&mut self, block: OwnedBlock<Block>) {
        let block = block.into_raw();
        // SAFETY: a token points at a live block of the arena.
        let flags = unsafe { block.as_ref() }.flags();
        flags.store(BlockFlagBits::new().order(flags.order()));

        self.attach(block);
        self.merge(block);
    }

    /// Splits the smallest free block above `order` down to `order`, and
    /// returns its first half, detached. Each split puts the second half
    /// in the free list one order down.
    fn split_until(&mut self, order: usize) -> Option<NonNull<Block>> {
        let (mut current, mut block) = (order + 1
            ..=BuddyOrder::MAX as usize)
            .find_map(|i| Some((i, self.pop(i)?)))?;

        while current > order {
            let (lhs, rhs) = self.split(block);
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
                (
                    block.as_ref().flags().order(),
                    buddy.as_ref().flags().order(),
                )
            };
            block = match buddy_order.cmp(&order) {
                Ordering::Less => buddy,
                Ordering::Equal => {
                    self.detach(block);
                    self.detach(buddy);
                    let merged = self.merge_pair(block, buddy);
                    self.attach(merged);
                    merged
                }
                Ordering::Greater => {
                    unreachable!("a buddy is never larger than its block")
                }
            };
        }
    }

    /// Splits `block`, a free block above order 0 in no free list, into
    /// its two halves one order down.
    fn split(
        &mut self,
        block: NonNull<Block>,
    ) -> (NonNull<Block>, NonNull<Block>) {
        // SAFETY: the allocator splits only free blocks of its arena.
        let page = unsafe { block.as_ref() };
        debug_assert!(
            // SAFETY: a free block belongs to the allocator.
            !unsafe { page.links().as_ref() }.is_linked(),
            "the block to split is in no free list"
        );
        let half = page
            .flags()
            .order()
            .prev()
            .expect("a block to split is above order 0");

        // The block's new order finds its buddy at that order.
        page.flags().modify(|bits| bits.set_order(half));
        let buddy = self
            .arena
            .buddy_of(block)
            .expect("the first half of a split block has a buddy");
        // SAFETY: `buddy_of` returns a live block of the arena.
        unsafe { buddy.as_ref() }
            .flags()
            .modify(|bits| bits.set_order(half));
        (block, buddy)
    }

    /// Merges `block` and `buddy`, free buddies of one order in no free
    /// list, into one block an order up.
    fn merge_pair(
        &mut self,
        block: NonNull<Block>,
        buddy: NonNull<Block>,
    ) -> NonNull<Block> {
        // SAFETY: the allocator merges only free blocks of its arena.
        let (block_page, buddy_page) =
            unsafe { (block.as_ref(), buddy.as_ref()) };
        let order = block_page.flags().order();
        debug_assert!(order == buddy_page.flags().order());
        debug_assert!(
            !block_page.flags().is_allocated()
                && !buddy_page.flags().is_allocated()
        );
        debug_assert!(
            // SAFETY: both blocks are free, so they belong to the
            // allocator.
            unsafe {
                !block_page.links().as_ref().is_linked()
                    && !buddy_page.links().as_ref().is_linked()
            },
            "the blocks to merge are in no free list"
        );

        let next_order = order
            .next()
            .expect("a block with a buddy is below the top order");
        let (l, r) = if block < buddy {
            (block_page, buddy_page)
        } else {
            (buddy_page, block_page)
        };
        l.flags().modify(|bits| bits.set_order(next_order));
        r.flags().modify(|bits| bits.set_order(BuddyOrder::None));
        NonNull::from_ref(l)
    }

    /// Detaches the first block of the free list of `order`.
    fn pop(&mut self, order: usize) -> Option<NonNull<Block>> {
        let first = self.freelist[order].first()?;
        let block = self
            .arena
            .at(first.get() as usize)
            .expect("a free list links only blocks of its arena");
        self.detach(block);
        Some(block)
    }

    /// Links a block in no free list into the free list of its order.
    fn attach(&mut self, block: NonNull<Block>) {
        let index = self.arena.index_of(block);
        // SAFETY: the allocator attaches only free blocks of its arena.
        let page = unsafe { block.as_ref() };
        let order = page.flags().order() as usize;
        // SAFETY: a free block belongs to the allocator, whose lock
        // `&mut self` holds.
        unsafe { page.links().as_mut() }
            .as_state_mut::<Detached>()
            .link(index, &mut self.freelist[order], self.arena);
    }

    /// Unlinks a block from its free list.
    fn detach(&mut self, block: NonNull<Block>) {
        // SAFETY: free lists link only free blocks of the arena.
        let page = unsafe { block.as_ref() };
        let order = page.flags().order() as usize;
        // SAFETY: a free block belongs to the allocator, whose lock
        // `&mut self` holds.
        unsafe { page.links().as_mut() }
            .as_state_mut::<Attached>()
            .unlink(&mut self.freelist[order], self.arena);
    }

    /// Marks a detached block as allocated.
    ///
    /// # Safety
    ///
    /// `block` must be a free block of the arena, in no free list.
    unsafe fn mark_allocated(&mut self, block: NonNull<Block>) {
        // SAFETY: the caller guarantees `block` is a free block of the
        // arena.
        let page = unsafe { block.as_ref() };
        debug_assert!(
            !page.flags().is_allocated()
                && !unsafe { page.links().as_ref() }.is_linked(),
            "{block:?} is free and in no free list"
        );
        page.flags().modify(|bits| bits.set_allocated(true));
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
