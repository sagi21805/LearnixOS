use core::{fmt::Debug, mem::ManuallyDrop, num::NonZero};

use common::constants::REGULAR_PAGE_SIZE;

use crate::alloc_pages;

pub union SlabCaches {
    pub generic4: ManuallyDrop<SlabCache<u32>>,
}

pub static SLABS: [SlabCaches; 1] = [SlabCaches {
    generic4: ManuallyDrop::new(SlabCache::new()),
}];

/// Preallocated object in the slab allocator.
///
/// When a slab is initialized, each position will include the index of the
/// next free object, when the object is allocated this index will be
/// overwrite by the objects data thus wasting no space on the free list.
pub union PreallocatedObject<T: 'static + Sized> {
    pub allocated: ManuallyDrop<T>,
    pub next_free_idx: Option<NonZero<u16>>,
}

impl<T> Debug for PreallocatedObject<T> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
pub struct SlabDescriptor<T: 'static + Sized> {
    /// The index in the objects array of the next free objet
    pub next_free_idx: usize,
    pub objects: &'static mut [PreallocatedObject<T>],
    pub next: Option<&'static mut SlabDescriptor<T>>,
}

impl<T> SlabDescriptor<T> {
    pub fn new(
        order: usize,
        next: Option<&'static mut SlabDescriptor<T>>,
    ) -> SlabDescriptor<T> {
        let address = unsafe { alloc_pages!(1 << order) };
        let objects = unsafe {
            core::slice::from_raw_parts_mut(
                address as *mut PreallocatedObject<T>,
                ((1 << order) * REGULAR_PAGE_SIZE) / size_of::<T>(),
            )
        };

        for (i, object) in objects.iter_mut().enumerate() {
            *object = PreallocatedObject {
                next_free_idx: Some(NonZero < i as u16 + 1),
            }
        }

        unsafe { objects.last().unwrap().next_free_idx }

        SlabDescriptor {
            next_free_idx: 0,
            objects,
            next,
        }
    }
}

#[derive(Debug)]
pub struct SlabCache<T: 'static + Sized> {
    // TODO ADD LOCK
    pub buddy_order: usize,
    pub partial: SlabDescriptor<T>,
    pub full: SlabDescriptor<T>,
    pub free: SlabDescriptor<T>,
}

impl<T> SlabCache<T> {
    pub const fn new() -> SlabCache<T> {
        unimplemented!()
    }
}

const trait SlabPosition {
    fn get_position() -> usize;
}
