use core::ptr::NonNull;

use crate::traits::Slab;

use super::descriptor::SlabDescriptor;

#[derive(Debug)]
pub struct SlabCache<T: Slab> {
    pub buddy_order: usize,
    pub free: Option<NonNull<SlabDescriptor<T>>>,
    pub partial: Option<NonNull<SlabDescriptor<T>>>,
    pub full: Option<NonNull<SlabDescriptor<T>>>,
}

#[rustfmt::skip]
impl const Default for SlabCache<()> {
    fn default() -> Self {
        SlabCache {
            buddy_order: 0,
            free: None,
            partial: None,
            full: None,
        }
    }
}

impl<T: Slab> SlabCache<T> {
    pub fn new(buddy_order: usize) -> SlabCache<T> {
        SlabCache {
            buddy_order,
            free: None,
            partial: None,
            full: None,
        }
    }

    pub unsafe fn as_unit(self) -> SlabCache<()> {
        unsafe { core::mem::transmute(self) }
    }

    /// Allocate a new slab descriptor, attaches it to the free slab list,
    /// and initialize it's page.
    ///
    /// TODO: THIS FUNCTION SHOULD MOVE TO THE SLAB ALLOCATOR AND GET A
    /// TYPE OF A SPECIFIC SLAB
    pub fn grow(&mut self) {
        // Allocate a new slab descriptor for this slab
        // let mut slab = unsafe {
        //     SLAB_ALLOCATOR.kmalloc::<SlabDescriptor<()>>().assign::<T>()
        // };

        // unsafe {
        //     *slab.as_mut() =
        //         SlabDescriptor::<T>::new(self.buddy_order, self.free)
        // }

        // self.take_ownership(slab);

        // self.free = Some(slab);
    }

    pub fn alloc(&mut self) -> NonNull<T> {
        if let Some(mut partial) = self.partial {
            let partial = unsafe { partial.as_mut() };

            let allocation = partial.alloc();

            if partial.next_free_idx.is_none() {
                self.partial = partial.next;
                partial.next = self.full;
                self.full = Some(NonNull::from_mut(partial));
            }
            return allocation;
        }
        if let Some(mut free) = self.free {
            let free = unsafe { free.as_mut() };

            let allocation = free.alloc();

            self.free = free.next;
            free.next = self.partial;
            self.partial = Some(NonNull::from_mut(free));

            return allocation;
        }

        todo!(
            "Handle cases where partial and free are full, and \
             allocation from the page allocator is needed."
        )
    }
    pub fn dealloc(&self, _ptr: NonNull<T>) { todo!() }
}
