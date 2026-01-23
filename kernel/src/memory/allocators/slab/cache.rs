use super::descriptor::SlabDescriptor;
use super::traits::{SlabCacheConstructor, SlabPosition};
use crate::memory::allocators::slab::SLAB_ALLOCATOR;
use crate::memory::unassigned::{AssignSlab, Unassigned};
use core::ptr::NonNull;

#[derive(Clone, Debug)]
pub struct SlabCache<T: 'static + Sized + SlabPosition> {
    pub buddy_order: usize,
    pub free: Option<NonNull<SlabDescriptor<T>>>,
    pub partial: Option<NonNull<SlabDescriptor<T>>>,
    pub full: Option<NonNull<SlabDescriptor<T>>>,
}

impl<T: SlabPosition> SlabCache<T> {
    pub fn as_unassigned(&self) -> &SlabCache<Unassigned> {
        unsafe { &*(self as *const _ as *const SlabCache<Unassigned>) }
    }

    pub fn as_unassigned_mut(&mut self) -> &mut SlabCache<Unassigned> {
        unsafe { &mut *(self as *mut _ as *mut SlabCache<Unassigned>) }
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
    pub fn dealloc(&self, _ptr: NonNull<T>) {
        todo!()
    }
}

impl SlabCache<Unassigned> {
    pub fn assign<T: SlabPosition>(&self) -> NonNull<SlabCache<T>> {
        unsafe {
            NonNull::new_unchecked(self as *const _ as *mut SlabCache<T>)
        }
    }
}

impl<T: SlabPosition> SlabCacheConstructor for SlabCache<T> {
    default fn new(buddy_order: usize) -> SlabCache<T> {
        let mut free = unsafe {
            SLAB_ALLOCATOR
                .slab_of::<SlabDescriptor<Unassigned>>()
                .as_mut()
                .alloc()
        };

        unsafe { *free.as_mut() = SlabDescriptor::new(buddy_order, None) }

        SlabCache {
            buddy_order,
            free: Some(free.assign::<T>()),
            partial: None,
            full: None,
        }
    }
}

impl SlabCacheConstructor for SlabCache<SlabDescriptor<Unassigned>> {
    fn new(buddy_order: usize) -> SlabCache<SlabDescriptor<Unassigned>> {
        let partial = SlabDescriptor::<SlabDescriptor<Unassigned>>::initial_descriptor(buddy_order);
        SlabCache {
            buddy_order,
            free: None,
            partial: Some(partial),
            full: None,
        }
    }
}
