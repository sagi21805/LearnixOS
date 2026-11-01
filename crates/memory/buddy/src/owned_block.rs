//! The right to write one allocated block.

use core::ptr::NonNull;

use crate::meta::BuddyBlock;

/// The right to write one allocated block, its body and the owner bits
/// of its flags.
///
/// [`BuddyAllocator::allocate_block`](crate::BuddyAllocator::allocate_block)
/// makes it, and
/// [`BuddyAllocator::deallocate_block`](crate::BuddyAllocator::deallocate_block)
/// takes it back. Anyone may read a block's flags, but outside buddy only
/// this token writes them.
///
/// ```compile_fail,E0624
/// use buddy::meta::BlockFlags;
///
/// fn write(flags: &BlockFlags) {
///     // Doesn't compile, because only an `OwnedBlock` writes a block's
///     // flags.
///     flags.modify(|bits| bits.set_owner(1));
/// }
/// ```
#[must_use = "dropping the token leaks its block"]
pub struct OwnedBlock<Block>(NonNull<Block>);

impl<Block: BuddyBlock> OwnedBlock<Block> {
    /// The block this token owns.
    pub fn as_ptr(&self) -> NonNull<Block> { self.0 }

    /// Replaces the owner bits of the block's flags with `owner`.
    pub fn set_owner(&mut self, owner: u8) {
        // SAFETY: a token points at a live block of its arena.
        unsafe { self.0.as_ref() }
            .flags()
            .modify(|bits| bits.set_owner(owner));
    }

    /// Gives the token up and returns its block, which
    /// [`from_raw`](Self::from_raw) turns back into a token.
    pub fn into_raw(self) -> NonNull<Block> { self.0 }

    /// The token for `block`.
    ///
    /// # Safety
    ///
    /// `block` must be a block the buddy allocator handed out and that
    /// isn't freed, and no other token for it may be in use.
    pub unsafe fn from_raw(block: NonNull<Block>) -> Self {
        debug_assert!(
            unsafe { block.as_ref() }.flags().is_allocated(),
            "an owned block is allocated"
        );
        Self(block)
    }
}
