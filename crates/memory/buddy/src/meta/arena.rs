//! The arena that holds every block's flags and [`BuddyLinks`].

use core::{ops::Range, ptr::NonNull};

use common::address_types::{PhysicalAddress, VirtualAddress};
use thiserror::Error;
use x86::{memory_map::MemoryMap, structures::paging::VirtualAddressExt};

use crate::meta::{BlockFlags, BuddyLinks, Regular};

#[derive(Debug, Error)]
pub enum BuddyError {
    #[error("Cannot find a buddy for a block that is BuddyOrder::MAX")]
    MaxOrder,
    #[error("Page is part of a larger order")]
    PageInLargerOrder,
    #[error("Cannot split a block that is BuddyOrder::MIN")]
    Unsplitable,
    #[error("The buddy of this block is outside of memory range")]
    BuddyOutOfRange,
    #[error("Page is not part of the arena")]
    PageNotInArena,
    #[error("The virtual address is not mapped")]
    AddressNotMapped,
}

/// One page's entry in the arena, its flags and the body that buddy's
/// links share with whatever owns the page.
pub trait BuddyBlock: Sized {
    /// The block's flags, which anyone may read.
    fn flags(&self) -> &BlockFlags;

    /// The block's free-list links.
    ///
    /// # Safety
    ///
    /// Buddy must own the block, so it's free and the buddy lock is held.
    unsafe fn links(&self) -> NonNull<BuddyLinks<Regular>>;

    /// The block whose links are at `links`.
    fn from_links(links: NonNull<BuddyLinks<Regular>>) -> NonNull<Self>;
}

pub trait BuddyArena<Block: BuddyBlock>: Sized {
    /// An arena of free order-0 blocks covering `mmap`, in no free list.
    fn new(mmap: &MemoryMap) -> Self;

    /// The physical memory that holds this arena's blocks.
    fn storage(&self) -> Range<PhysicalAddress>;

    /// Returns an iterator over all blocks in this arena.
    fn iter(&self) -> impl ExactSizeIterator<Item = NonNull<Block>>;

    /// Returns the buddy of a block.
    fn buddy_of(
        &self,
        block: NonNull<Block>,
    ) -> Result<NonNull<Block>, BuddyError>;

    /// Returns the physical allocated address by this block.
    ///
    /// The first block of Order1 for example allocates 0..4096
    fn address_of(&self, block: NonNull<Block>) -> PhysicalAddress;

    /// Return the corresponding block in the arena for a given physical
    /// address, if one exists.
    fn page_with_address(
        &self,
        address: PhysicalAddress,
    ) -> Result<NonNull<Block>, BuddyError>;

    /// The block holding the page that `address` maps to.
    fn page_with_virtual_address(
        &self,
        address: VirtualAddress,
    ) -> Result<NonNull<Block>, BuddyError> {
        let address =
            address.translate().ok_or(BuddyError::AddressNotMapped)?;
        self.page_with_address(address)
    }

    /// The buddy of `block` when it is free, or `None` when it is
    /// allocated or `block` has no buddy.
    ///
    /// `block` must head a range of this arena.
    fn free_buddy_of(
        &self,
        block: NonNull<Block>,
    ) -> Option<NonNull<Block>> {
        match self.buddy_of(block) {
            // SAFETY: `buddy_of` returns a live block of this arena.
            Ok(buddy) => {
                (!unsafe { buddy.as_ref() }.flags().is_allocated())
                    .then_some(buddy)
            }
            Err(BuddyError::BuddyOutOfRange | BuddyError::MaxOrder) => {
                None
            }
            // SAFETY: `block` heads a range, so `buddy_of` fails only
            // when the block has no buddy.
            Err(_) => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    /// Split a block into two smaller blocks of previous order.
    ///
    /// The block must be in no free list.
    fn split(
        &self,
        block: NonNull<Block>,
    ) -> Result<(NonNull<Block>, NonNull<Block>), BuddyError>;

    /// Merge two blocks with the same order to a next order block.
    ///
    /// The two blocks must be buddies, have the same order, be free, and
    /// be in no free list.
    fn merge(
        &self,
        block: NonNull<Block>,
        buddy: NonNull<Block>,
    ) -> Result<NonNull<Block>, BuddyError>;

    // /// Detach a block from the middle of the arena, returning the
    // detached /// block.
    // fn detach_mid(&self, block: NonNull<Block>) -> NonNull<Block>;

    /// Returns the block nth block of the arena, if one exists.
    fn at(&self, n: usize) -> Option<NonNull<Block>>;

    /// Returns the (section index, section offset) of the given page.
    ///
    /// A section is a contiguous range of pages aligned to
    /// [`BuddyOrder::MAX`].
    ///
    /// # Safety
    ///
    /// The page must be contained within the arena's memory range.
    unsafe fn section_index_of(
        &self,
        block: NonNull<Block>,
    ) -> (usize, usize);
}
