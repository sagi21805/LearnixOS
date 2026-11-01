mod arena;
mod list;
mod state;

pub use arena::*;
pub use list::*;
pub use state::*;

use core::{
    fmt::Debug,
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{AtomicU64, Ordering},
};

use bitfields::bitfields;
use common::{enums::BuddyOrder, page_index::PageIndex};
use typestate_groups::typestate;

#[bitfields]
pub struct BlockFlagBits {
    #[flag(flag_type = BuddyOrder)]
    pub order: B4,
    pub allocated: B1,
    #[flag(r)]
    pub reserved: B51,
    /// Bits the block's owner gives its own meaning. Buddy zeroes them
    /// when the block comes back.
    pub owner: B8,
}

/// A block's flags, which anyone holding the arena may read and only the
/// block's owner may write. Buddy owns a free block, and whoever
/// allocated a block owns it until they free it.
///
/// Each block has one writer at a time, so a write is a load, a change
/// and a store, with no read-modify-write instruction. The word is atomic
/// so a read that overlaps the owner's write is a race on a value, not
/// undefined behaviour.
///
/// Every access is `Relaxed`, because no flag publishes other memory.
/// Locks and the handoff of a block in `allocate` and `deallocate` order
/// every access to a block's body. Code that reads a flag without a lock
/// and then reads the body because of it needs an `Acquire` load, paired
/// with a `Release` store of that flag.
#[repr(transparent)]
pub struct BlockFlags(AtomicU64);

impl BlockFlags {
    /// A word holding `bits`.
    pub const fn new(bits: BlockFlagBits) -> BlockFlags {
        BlockFlags(AtomicU64::new(u64::from(bits)))
    }

    /// The flags as they are now.
    pub fn load(&self) -> BlockFlagBits {
        BlockFlagBits::from(self.0.load(Ordering::Relaxed))
    }

    /// Replaces every flag with `bits`. Only the block's owner calls this.
    pub fn store(&self, bits: BlockFlagBits) {
        self.0.store(u64::from(bits), Ordering::Relaxed);
    }

    /// Changes the flags with `f`. Only the block's owner calls this.
    pub fn modify(&self, f: impl FnOnce(&mut BlockFlagBits)) {
        let mut bits = self.load();
        f(&mut bits);
        self.store(bits);
    }

    /// The order of the range this block heads, or [`BuddyOrder::None`]
    /// when it lies inside a larger block.
    pub fn order(&self) -> BuddyOrder { self.load().get_order() }

    /// Whether this block is handed out.
    pub fn is_allocated(&self) -> bool { self.load().is_allocated() }

    /// The bits the block's owner keeps here.
    pub fn owner(&self) -> u8 { self.load().get_owner() }

    /// Replaces the owner's bits with `owner`. Only the block's owner
    /// calls this.
    pub fn set_owner(&self, owner: u8) {
        self.modify(|bits| bits.set_owner(owner));
    }
}

impl Debug for BlockFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.load().fmt(f)
    }
}

/// The free-list links of a block, in a free list or not.
///
/// The state lives only in a `PhantomData` and the struct is `repr(C)`,
/// so every state has the same layout.
#[typestate]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BuddyLinks<S: BuddyState> {
    /// The block after this one in its free list.
    pub next: Option<PageIndex>,
    pub prev: Prev,
    _state: PhantomData<S>,
}

const _: () = assert!(size_of::<BuddyLinks<Regular>>() == 8);

/// Where a block sits in its free list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Prev {
    /// In no free list.
    Unlinked,
    /// First in its list, which the free list of its order points at.
    First,
    /// Right after the block at this index.
    After(PageIndex),
}

const _: () = assert!(size_of::<Prev>() == 4);

impl BuddyLinks<Regular> {
    /// The links of a block in no free list.
    pub const fn new() -> BuddyLinks<Regular> {
        BuddyLinks {
            next: None,
            prev: Prev::Unlinked,
            _state: PhantomData,
        }
    }
}

impl Default for BuddyLinks<Regular> {
    fn default() -> Self { Self::new() }
}

impl<S: BuddyState> BuddyLinks<S> {
    /// Whether this block is in a free list.
    pub fn is_linked(&self) -> bool { self.prev != Prev::Unlinked }

    /// These links, as links in state `To`.
    ///
    /// `BuddyLinks<S> -> BuddyLinks<To>`
    pub(crate) fn as_state_mut<To: BuddyState>(
        &mut self,
    ) -> &mut BuddyLinks<To> {
        // SAFETY: every state has the same layout, and `&mut self` makes
        // the new reference the only one.
        unsafe { NonNull::from_mut(self).cast().as_mut() }
    }
}

impl<S: BuddyStateGroupMember<Nodes>> Debug for BuddyLinks<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("BuddyLinks");

        ds.field("next", &self.next).field("prev", &self.prev);

        #[cfg(debug_assertions)]
        ds.field("self_ptr", &NonNull::from_ref(self));

        ds.finish()
    }
}

/// A free list, as the index of its first block.
#[repr(transparent)]
pub struct FreeList(Option<PageIndex>);

impl FreeList {
    /// A list with no blocks.
    pub const fn empty() -> FreeList { FreeList(None) }

    /// The first block of the list.
    pub fn first(&self) -> Option<PageIndex> { self.0 }

    /// Makes `first` the first block of the list.
    fn set_first(&mut self, first: Option<PageIndex>) { self.0 = first; }
}

impl Debug for FreeList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("FreeList").field(&self.0).finish()
    }
}
