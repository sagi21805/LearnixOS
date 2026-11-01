#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![allow(incomplete_features)]
#![feature(min_specialization)]
#![feature(ptr_alignment_type)]

mod init;
pub mod meta;

use core::{
    alloc::{AllocError, Allocator, Layout},
    cmp::Ordering,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
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

pub struct BuddyAllocator<Arena, Block>
where
    Block: BuddyBlock,
    Arena: BuddyArena<Block>,
{
    inner: SpinMutex<BuddyAllocatorInner<Arena, Block>>,
    _pinned: PhantomPinned,
}

impl<Arena, Block> BuddyAllocator<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    pub fn new(memory_map: &MemoryMap) -> BuddyAllocator<Arena, Block> {
        BuddyAllocator {
            inner: SpinMutex::new(BuddyAllocatorInner {
                arena: Arena::new(memory_map),
                freelist: [const { FreeList::empty() };
                    BuddyOrder::MAX as usize + 1],
                _block: PhantomData,
            }),
            _pinned: PhantomPinned,
        }
    }

    /// Builds the free lists from the arena.
    ///
    /// Reserves the memory that isn't free, links every free page into the
    /// order-0 list, and merges buddies from there. The lists point into
    /// `self`, so it takes a pinned `self` to keep the allocator from
    /// moving.
    pub fn initialize(self: Pin<&mut Self>, mmap: &MemoryMap) {
        self.inner.lock().initialize(mmap);
    }

    /// Locks the allocator until the guard is dropped.
    ///
    /// Hold this lock while writing the metadata of a block you allocated,
    /// such as a slab descriptor. Freeing a block reads the metadata of
    /// its buddy to check whether the two can merge, and that buddy may be
    /// your block. The lock keeps that read from running in the middle of
    /// your write.
    pub fn lock(
        &self,
    ) -> MutexGuard<'_, BuddyAllocatorInner<Arena, Block>, Spin> {
        self.inner.lock()
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
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.inner.lock().allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { self.inner.lock().deallocate(ptr, layout) }
    }
}

/// An arena and the free lists that link its blocks, reached through
/// [`BuddyAllocator::lock`].
pub struct BuddyAllocatorInner<Arena, Block> {
    arena: Arena,
    freelist: [FreeList; BuddyOrder::MAX as usize + 1],
    _block: PhantomData<Block>,
}

impl<Arena, Block> BuddyAllocatorInner<Arena, Block>
where
    Arena: BuddyArena<Block>,
    Block: BuddyBlock,
{
    /// Takes a block of the smallest order that fits `layout`, and
    /// returns all of it, through the higher-half map of physical memory.
    pub fn allocate(
        &mut self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let order = order_of(layout).ok_or(AllocError)?;

        let block = self
            .pop(order)
            .or_else(|| self.split_until(order))
            .ok_or(AllocError)?;
        // SAFETY: `pop` and `split_until` return a free block of the
        // arena, which they detached.
        unsafe { self.mark_allocated(block) };

        let start =
            self.arena.address_of(block).translate().as_non_null::<u8>();
        Ok(NonNull::slice_from_raw_parts(
            start,
            REGULAR_PAGE_SIZE << order,
        ))
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

    /// Puts the block at `ptr` back in its free list, and merges it with
    /// its free buddies.
    ///
    /// # Safety
    ///
    /// `ptr` and `layout` must come from one call to `allocate`, and the
    /// block must not have been freed since.
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let order = order_of(layout);
        debug_assert!(order.is_some(), "`layout` came from `allocate`");
        // SAFETY: the caller guarantees `layout` is the one `allocate`
        // accepted, so it has an order.
        let order = unsafe { order.unwrap_unchecked() };

        let block = self
            .block_of(ptr)
            .expect("an allocated block lies in the arena");

        // SAFETY: `block_of` returns a live block of the arena.
        let flags = unsafe { block.as_ref() }.flags();
        debug_assert!(flags.is_allocated(), "{block:?} is allocated");
        flags.store(
            BlockFlagBits::new().order(BuddyOrder::from(order as u8)),
        );

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
        let block =
            Block::from_links(self.freelist[order].first()?.cast());
        self.detach(block);
        Some(block)
    }

    /// Links a block in no free list into the free list of its order.
    fn attach(&mut self, block: NonNull<Block>) {
        // SAFETY: the allocator attaches only free blocks of its arena.
        let block = unsafe { block.as_ref() };
        let order = block.flags().order() as usize;
        // SAFETY: a free block belongs to the allocator, whose lock
        // `&mut self` holds.
        unsafe { block.links().as_mut() }
            .as_state_mut::<Detached>()
            .link(self.freelist[order].head_mut());
    }

    /// Unlinks a block from its free list.
    fn detach(&mut self, block: NonNull<Block>) {
        // SAFETY: free lists link only free blocks of the arena, which
        // belong to the allocator.
        unsafe { block.as_ref().links().as_mut() }
            .as_state_mut::<Attached>()
            .unlink();
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
