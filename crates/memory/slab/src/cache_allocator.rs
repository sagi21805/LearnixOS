use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use buddy::meta::{BuddyArena, BuddyBlock};
use common::address_types::VirtualAddress;
use x86::structures::paging::VirtualAddressExt;

use crate::{
    SlabAllocator,
    traits::{Slab, SlabBlock},
};

/// Allocates `T`s from their slab cache.
///
/// Freeing locks the buddy arena before the slab array, and every other
/// path that takes both locks must take them in that order.
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

    /// Whether objects of this cache can hold `layout`.
    fn fits(layout: Layout) -> bool {
        layout.size() <= size_of::<T>()
            && layout.align() <= T::OBJECT_ALIGNMENT
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

// SAFETY: every handle to one `SlabAllocator` reaches the same caches, so
// a block from one copy can be freed through another. Blocks stay valid
// until `deallocate`, because the slab never hands a slot out twice.
unsafe impl<T: Slab, Block, Arena> Allocator
    for CacheAllocator<'_, T, Block, Arena>
where
    Block: BuddyBlock + SlabBlock,
    Arena: BuddyArena<Block> + 'static,
{
    /// Takes one object, and returns all of it, so any size up to
    /// `size_of::<T>()` frees it again.
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if !Self::fits(layout) {
            return Err(AllocError);
        }

        let object =
            self.slab.slab_arena.lock().cache::<T>().take_object();
        Ok(NonNull::slice_from_raw_parts(object.cast(), size_of::<T>()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        debug_assert!(Self::fits(layout), "the layout fits in a `T`");
        let object = ptr.cast::<T>();

        let arena_lock = self.slab.buddy_arena.lock();
        let mut slab_lock = self.slab.slab_arena.lock();

        let address = VirtualAddress::from(ptr.addr().get());
        let mut page = arena_lock
            .page_with_address(
                address
                    .translate()
                    .expect("an allocated object is mapped"),
            )
            .expect("the buddy arena covers every mapped page");

        // SAFETY: `page` is a live page of the locked buddy arena.
        let page = unsafe { page.as_mut() };
        // SAFETY: the page holds `object`, which belongs to a slab of `T`,
        // so the page's descriptor is a `T` descriptor.
        let descriptor = unsafe { page.slab_descriptor_mut::<T>() };
        let idx = descriptor
            .objects
            .index_of(object)
            .expect("an allocated object is a slot of its page's slab");

        // SAFETY: the caller guarantees `object` is allocated, and its
        // slab sits in one of the cache's lists.
        unsafe { slab_lock.cache::<T>().return_object(idx, descriptor) };

        drop(arena_lock);
        drop(slab_lock);
    }
}
