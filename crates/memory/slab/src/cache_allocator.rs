use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use buddy::meta::{BuddyArena, BuddyBlock};
use common::address_types::VirtualAddress;

use crate::{
    SlabAllocator,
    traits::{Slab, SlabBlock},
};

/// Allocates `T`s from their slab cache.
///
/// Freeing locks the buddy arena before the slab array. Every other path
/// that takes both locks must take them in that order.
///
/// ```ignore
/// let inode = Box::new_in(Inode::default(), SLAB.cache::<Inode>());
/// ```
pub struct CacheAllocator<'a, T: Slab, Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    slab: &'a SlabAllocator<Block, Arena>,
    _object: PhantomData<fn() -> T>,
}

impl<'a, T: Slab, Block, Arena> CacheAllocator<'a, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    pub(crate) const fn new(
        slab: &'a SlabAllocator<Block, Arena>,
    ) -> Self {
        Self {
            slab,
            _object: PhantomData,
        }
    }
}

impl<T: Slab, Block, Arena> Clone for CacheAllocator<'_, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    fn clone(&self) -> Self { *self }
}

impl<T: Slab, Block, Arena> Copy for CacheAllocator<'_, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
}

// SAFETY: every handle reaches the same caches, and a slot is never handed
// out twice.
unsafe impl<T: Slab, Block, Arena> Allocator
    for CacheAllocator<'_, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    /// Allocates one object, and returns all of it, so any size up to
    /// `size_of::<T>()` frees it again.
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if !T::fits(layout) {
            return Err(AllocError);
        }

        // The slab lock is released at the end of this statement, before
        // `new_slab` takes the buddy lock.
        let object =
            self.slab.slab_arena.lock().cache::<T>().try_allocate();
        let object = object.unwrap_or_else(|| {
            let slab = self.slab.new_slab::<T>();
            self.slab
                .slab_arena
                .lock()
                .cache::<T>()
                .allocate_and_link(slab)
        });
        Ok(NonNull::slice_from_raw_parts(object.cast(), size_of::<T>()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        debug_assert!(T::fits(layout), "the layout fits in a `T`");
        let object = ptr.cast::<T>();

        let arena = self.slab.buddy_arena.lock();
        let mut caches = self.slab.slab_arena.lock();

        let address = VirtualAddress::from(object.addr().get());
        let mut page = arena
            .page_with_virtual_address(address)
            .expect("an allocated object's page is in the buddy arena");
        // SAFETY: the page holds a `T` object, so its descriptor is a `T`
        // one.
        let slab = unsafe { page.as_mut().slab_descriptor_mut::<T>() };
        let idx = slab
            .objects
            .index_of(object)
            .expect("an allocated object is a slot of its page's slab");

        // SAFETY: an allocated object's slab is in one of the cache's
        // lists.
        unsafe { caches.cache::<T>().deallocate_object(idx, slab) };
    }
}
