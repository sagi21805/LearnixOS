//! The arena that holds every block's [`BuddyMeta`].

use core::ptr::NonNull;

use common::address_types::{PhysicalAddress, VirtualAddress};
use thiserror::Error;
use x86::{memory_map::MemoryMap, structures::paging::VirtualAddressExt};

use crate::meta::{BuddyMeta, Head, Regular};

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

pub const trait BuddyBlock: Sized {
    fn meta(&self) -> &BuddyMeta<Regular>;

    fn meta_mut(&mut self) -> &mut BuddyMeta<Regular>;

    fn from_meta(meta: NonNull<BuddyMeta<Regular>>) -> NonNull<Self>;
}

pub trait BuddyArena<Block: BuddyBlock>: Sized {
    fn new(mmap: &MemoryMap, head: &mut BuddyMeta<Head>) -> Self;

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

    /// Split a block into two smaller blocks of previous order.
    fn split(
        &self,
        block: NonNull<Block>,
    ) -> Result<(NonNull<Block>, NonNull<Block>), BuddyError>;

    /// Merge two blocks with the same order to a next order block.
    ///
    /// The two blocks must be buddies, have the same order, and be free.
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
