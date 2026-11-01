use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use buddy::meta::{BuddyArena, BuddyBlock};

use crate::{
    SlabAllocator, SlabArena,
    traits::{Slab, SlabBlock},
};

/// Allocates `T`s from their slab cache.
///
/// `allocate` and `deallocate` hold the buddy lock and then the slab lock
/// for their whole run, because the buddy allocator reads the metadata of
/// a slab's first page when it frees that page's buddy. Every path takes
/// the two in that order, so they can't deadlock.
///
/// ```ignore
/// let inode = Box::new_in(Inode::default(), SLAB.cache::<Inode>());
/// ```
pub struct CacheAllocator<'a, T: Slab, Block, Arena>
where
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
{
    slab: &'a SlabAllocator<Block, Arena>,
    _object: PhantomData<fn() -> T>,
}

impl<'a, T: Slab, Block, Arena> CacheAllocator<'a, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
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
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
{
    fn clone(&self) -> Self { *self }
}

impl<T: Slab, Block, Arena> Copy for CacheAllocator<'_, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
{
}

// SAFETY: every handle reaches the same caches, and a slot is never handed
// out twice.
unsafe impl<T: Slab, Block, Arena> Allocator
    for CacheAllocator<'_, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
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

        let mut buddy = self.slab.buddy.lock();
        let mut caches = self.slab.caches.lock();
        let cache = caches.cache::<T>();
        let object = match cache.try_allocate(buddy.arena()) {
            Some(object) => object,
            None => {
                let mut free = NonNull::from_mut(
                    SlabAllocator::new_slab::<T>(&mut buddy)?,
                );
                // SAFETY: the new slab is in no list, so nothing else
                // reaches it while the buddy lock is held.
                cache.allocate_and_link(buddy.arena(), unsafe {
                    free.as_mut()
                })
            }
        };
        Ok(NonNull::slice_from_raw_parts(object.cast(), size_of::<T>()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        debug_assert!(T::fits(layout), "the layout fits in a `T`");
        let object = ptr.cast::<T>();

        // A slab is one buddy block, which is aligned to its size, so its
        // first page is `ptr` rounded down to the slab's size.
        let offset = ptr.addr().get() % T::SLAB_LAYOUT.size();
        // SAFETY: the slab's first page starts `offset` bytes before
        // `ptr`, in the same pages.
        let page = unsafe { ptr.byte_sub(offset) };

        let buddy = self.slab.buddy.lock();
        let mut caches = self.slab.caches.lock();
        let page = buddy
            .block_of(page)
            .expect("an allocated object's page is in the buddy arena");
        // SAFETY: the page holds a `T` object, so its descriptor is a `T`
        // one.
        let slab = unsafe { page.as_ref().slab_descriptor_mut::<T>() };
        let arena = buddy.arena();
        let idx = arena
            .objects_of(slab)
            .index_of(object)
            .expect("an allocated object is a slot of its page's slab");

        // SAFETY: an allocated object's slab is in one of the cache's
        // lists.
        unsafe { caches.cache::<T>().deallocate_object(arena, idx, slab) };
    }
}
