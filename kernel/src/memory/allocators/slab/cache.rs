use super::descriptor::SlabDescriptor;
use super::slab_of;
use super::traits::{SlabCacheConstructor, SlabPosition};
use crate::memory::page_descriptor::Unassigned;
use core::ptr::NonNull;

#[derive(Debug)]
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

    pub fn alloc(&self, _obj: T) -> NonNull<T> {
        unimplemented!()
    }
    pub fn dealloc(&self, _obj: NonNull<T>) {
        unimplemented!()
    }
}

impl SlabCache<Unassigned> {
    pub fn assign<T: SlabPosition>(&self) -> &SlabCache<T> {
        unsafe { &*(self as *const _ as *const SlabCache<T>) }
    }

    pub fn assign_mut<T: SlabPosition>(&mut self) -> &mut SlabCache<T> {
        unsafe { &mut *(self as *mut _ as *mut SlabCache<T>) }
    }
}

impl<T: SlabPosition> SlabCacheConstructor for SlabCache<T> {
    default fn new(buddy_order: usize) -> SlabCache<T> {
        let free = slab_of::<SlabDescriptor<Unassigned>>()
            .alloc(SlabDescriptor::new(buddy_order, None));

        SlabCache {
            buddy_order,
            free: Some(unsafe { free.as_ref().assign::<T>() }),
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
