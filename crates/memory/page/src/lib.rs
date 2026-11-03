#![no_std]
#![feature(const_trait_impl)]

pub mod arena;
mod placement;

use core::{
    cell::UnsafeCell, fmt::Debug, mem::ManuallyDrop, ptr::NonNull,
};

use buddy::meta::{
    BlockFlagBits, BlockFlags, BuddyBlock, BuddyLinks, Regular,
};

use slab::{
    descriptor::{SlabDescriptor, Used},
    traits::{Slab, SlabBlock, Unknown},
};

/// The arena's entry for one physical page.
///
/// The flags are the part anyone may read. The body is the part only the
/// page's owner may touch, buddy's links while the page is free, or its
/// slab descriptor while a slab owns it.
#[repr(C)]
pub struct Page {
    flags: BlockFlags,
    body: UnsafeCell<PageBody>,
}

const _: () = assert!(size_of::<Page>() == 16);

/// What the owner of a page keeps in it.
union PageBody {
    buddy: BuddyLinks<Regular>,
    slab: ManuallyDrop<SlabDescriptor<Unknown, Used>>,
}

// SAFETY: anyone may read the flags, as they're atomic. Only the page's
// owner reaches the body, and ownership changes hands only inside buddy's
// `allocate` and `deallocate`, under the buddy lock.
unsafe impl Sync for Page {}

impl Page {
    /// A page with `bits`, in no free list.
    pub const fn new(bits: BlockFlagBits) -> Page {
        Page {
            flags: BlockFlags::new(bits),
            body: UnsafeCell::new(PageBody {
                buddy: BuddyLinks::new(),
            }),
        }
    }
}

impl Debug for Page {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("Page");
        ds.field("flags", &self.flags);
        if !self.flags.is_allocated() {
            // SAFETY: a free page's body holds its links.
            ds.field("links", unsafe { self.links().as_ref() });
        }
        ds.finish()
    }
}

impl BuddyBlock for Page {
    fn flags(&self) -> &BlockFlags { &self.flags }

    unsafe fn links(&self) -> NonNull<BuddyLinks<Regular>> {
        debug_assert!(
            !self.flags.is_allocated(),
            "only a free page has links"
        );
        // SAFETY: the body sits inside `self`, so its address isn't null.
        unsafe {
            NonNull::new_unchecked(&raw mut (*self.body.get()).buddy)
        }
    }
}

impl SlabBlock for Page {
    unsafe fn slab_descriptor<T: Slab>(&self) -> &SlabDescriptor<T, Used> {
        // SAFETY: a descriptor's layout doesn't depend on `T`, and the
        // caller guarantees this page holds a slab of `T`.
        unsafe { &*(&raw const (*self.body.get()).slab).cast() }
    }

    unsafe fn slab_descriptor_mut<T: Slab>(
        &self,
    ) -> &mut SlabDescriptor<T, Used> {
        // SAFETY: a descriptor's layout doesn't depend on `T`, and the
        // caller owns this page as a slab of `T`.
        unsafe { &mut *(&raw mut (*self.body.get()).slab).cast() }
    }
}
