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
use common::enums::BuddyOrder;
use typestate_groups::typestate;

/// The link from a node to the first free block after it.
pub type Next = Option<NonNull<BuddyLinks<Attached>>>;

/// The `prev` of a block in no free list. No link lives at this address,
/// so a real `prev` never equals it.
pub const UNLINKED: NonNull<Next> = NonNull::dangling();

#[bitfields]
pub struct BlockFlagBits {
    #[flag(flag_type = BuddyOrder)]
    pub order: B4,
    pub allocated: B1,
    #[flag(r)]
    pub reserved: B59,
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
    pub next: Next,
    /// The `next` link that points at this block, or [`UNLINKED`] while
    /// it is in no free list.
    pub prev: NonNull<Next>,
    _state: PhantomData<S>,
}

impl BuddyLinks<Regular> {
    /// The links of a block in no free list.
    pub const fn new() -> BuddyLinks<Regular> {
        BuddyLinks {
            next: None,
            prev: UNLINKED,
            _state: PhantomData,
        }
    }
}

impl Default for BuddyLinks<Regular> {
    fn default() -> Self { Self::new() }
}

impl<S: BuddyState> BuddyLinks<S> {
    /// Whether this block is in a free list.
    pub fn is_linked(&self) -> bool { self.prev != UNLINKED }

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

/// A free list, as the link to its first block.
///
/// Blocks in the list point back at this link, so a list must not move
/// while it holds blocks.
#[repr(transparent)]
pub struct FreeList(Next);

const _: () = assert!(size_of::<FreeList>() == size_of::<Next>());

impl FreeList {
    /// A list with no blocks.
    pub const fn empty() -> FreeList { FreeList(None) }

    /// The first block of the list.
    pub fn first(&self) -> Next { self.0 }

    /// The link to the first block, which a block links in after.
    pub fn head_mut(&mut self) -> &mut Next { &mut self.0 }
}

impl Debug for FreeList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list_fmt = f.debug_list();

        let mut next = self.0;
        let mut i = 0;
        while let Some(n) = next {
            list_fmt.entry(unsafe { n.as_ref() });
            next = unsafe { n.as_ref().next };
            if i == 32 {
                return list_fmt.finish();
            }
            i += 1;
        }

        list_fmt.finish()
    }
}

// SAFETY: a node's links point only into its allocator's arena and free
// lists, which the allocator's locks guard.
unsafe impl<S: BuddyState> Send for BuddyLinks<S> {}

// SAFETY: a free list links only blocks of its allocator's arena, which
// the allocator's locks guard.
unsafe impl Send for FreeList {}
