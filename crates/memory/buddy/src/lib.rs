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

use libk::println;
use sync::mutex::SpinMutex;
use x86::memory_map::MemoryMap;

use common::{
    address_types::{Address, PhysicalAddress},
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    enums::BuddyOrder,
};

use typestate_groups::Isomorphic;

use crate::meta::{
    Attached, BuddyArena, BuddyBlock, Detached, FreeList, Link, Unlink,
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

    /// Puts the block back in its free list, and merges it with its free
    /// buddies.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let order = order_of(layout);
        debug_assert!(order.is_some(), "`layout` came from `allocate`");
        // SAFETY: the caller guarantees `layout` is the one `allocate`
        // accepted, so it has an order.
        let order = unsafe { order.unwrap_unchecked() };

        let mut inner = self.inner.lock();
        let mut block = inner
            .arena
            .page_with_address(PhysicalAddress::from(ptr.addr().get()))
            .expect("an allocated block lies in the arena");

        // SAFETY: `block` is the allocated block at `ptr`, which nothing
        // else uses once it is freed.
        let meta = unsafe { block.as_mut() }.meta_mut();
        debug_assert!(meta.flags.is_allocated(), "{block:?} is allocated");
        meta.flags.set_order(BuddyOrder::from(order as u8));
        meta.flags.set_allocated(false);

        inner.attach(block);
        inner.merge(block);
    }
}

/// An arena and the free lists that link its blocks.
struct BuddyAllocatorInner<Arena, Block> {
    arena: Arena,
    freelist: [FreeList; BuddyOrder::MAX as usize + 1],
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
        let block = Block::from_meta(self.freelist[order].first()?.cast());
        self.detach(block);
        Some(block)
    }

    /// Links a block in no free list into the free list of its order.
    fn attach(&mut self, mut block: NonNull<Block>) {
        // SAFETY: the allocator attaches only live blocks of its arena.
        let meta = unsafe { block.as_mut() }.meta_mut();
        let order = meta.flags.get_order() as usize;
        meta.cast_state_mut::<Detached>()
            .link(self.freelist[order].head_mut());
    }

    /// Unlinks a block from its free list.
    fn detach(&mut self, mut block: NonNull<Block>) {
        // SAFETY: free lists link only live blocks of the arena.
        let meta = unsafe { block.as_mut() }.meta_mut();
        meta.cast_state_mut::<Attached>().unlink();
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
            !meta.flags.is_allocated() && !meta.is_linked(),
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
