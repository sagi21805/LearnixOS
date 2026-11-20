use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    SlabAllocator, SlabArena, SlabSystem, cache::CachedIn,
    traits::SlabBlock,
};

/// Allocates `T`s from their slab cache.
///
/// No path holds the buddy lock and the slab lock together. Buddy reads
/// only the shared flags of a page it doesn't own, and a slab's
/// descriptor sits in its page's body, which only the slab reaches. The
/// slab lock alone guards the descriptors and their lists.
///
/// ```ignore
/// let inode = Box::new_in(Inode::default(), SLAB.cache::<Inode>());
/// ```
pub struct CacheAllocator<'a, T: CachedIn<S::Caches>, S: SlabSystem> {
    slab: &'a SlabAllocator<S>,
    _object: PhantomData<fn() -> T>,
}

impl<'a, T: CachedIn<S::Caches>, S: SlabSystem> CacheAllocator<'a, T, S> {
    pub(crate) const fn new(slab: &'a SlabAllocator<S>) -> Self {
        Self {
            slab,
            _object: PhantomData,
        }
    }
}

impl<T: CachedIn<S::Caches>, S: SlabSystem> Clone
    for CacheAllocator<'_, T, S>
{
    fn clone(&self) -> Self { *self }
}

impl<T: CachedIn<S::Caches>, S: SlabSystem> Copy
    for CacheAllocator<'_, T, S>
{
}

// SAFETY: every handle reaches the same caches, and a slot is never handed
// out twice.
unsafe impl<T: CachedIn<S::Caches>, S: SlabSystem> Allocator
    for CacheAllocator<'_, T, S>
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

        let arena = self.slab.buddy.arena();
        let cached =
            self.slab.caches.lock().cache::<T>().try_allocate(arena);
        let object = match cached {
            Some(object) => object,
            None => {
                let free = self.slab.new_slab::<T>()?;
                self.slab
                    .caches
                    .lock()
                    .cache::<T>()
                    .allocate_and_link(arena, free)
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

        let page =
            self.slab.buddy.block_of(page).expect(
                "an allocated object's page is in the buddy arena",
            );

        let mut caches = self.slab.caches.lock();
        // SAFETY: the page holds a `T` object, so its descriptor is a `T`
        // one, and the slab lock keeps the lists off it.
        let slab = unsafe { page.as_ref().slab_descriptor_mut::<T>() };
        let arena = self.slab.buddy.arena();
        let idx = arena
            .objects_of(slab)
            .index_of(object)
            .expect("an allocated object is a slot of its page's slab");

        // SAFETY: an allocated object's slab is in one of the cache's
        // lists.
        unsafe { caches.cache::<T>().deallocate_object(arena, idx, slab) };
    }
}
