use core::{fmt::Debug, mem::ManuallyDrop};

use common::{constants::REGULAR_PAGE_SIZE, write_volatile};

use nonmax::NonMaxU16;

use crate::{alloc_pages, memory::page_descriptor::Unassigned};

pub union SlabCaches {
    pub generic4: ManuallyDrop<SlabCache<u32>>,
    pub slab_descriptor: ManuallyDrop<SlabDescriptor<Unassigned>>,
    pub slab_cache: ManuallyDrop<SlabCache<Unassigned>>,
}

pub static SLABS: [SlabCaches; 1] = [SlabCaches {
    slab_descriptor: ManuallyDrop::new(SlabCache::new()),
}];

/// Preallocated object in the slab allocator.
///
/// When a slab is initialized, each position will include the index of the
/// next free object, when the object is allocated this index will be
/// overwrite by the objects data thus wasting no space on the free list.
pub union PreallocatedObject<T: 'static + Sized> {
    pub allocated: ManuallyDrop<T>,
    pub next_free_idx: Option<NonMaxU16>,
}

impl<T> Debug for PreallocatedObject<T> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
pub struct SlabDescriptor<T: 'static + Sized> {
    /// The index in the objects array of the next free objet
    pub next_free_idx: Option<NonMaxU16>,
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
                next_free_idx: Some(unsafe {
                    NonMaxU16::new_unchecked(i as u16 + 1)
                }),
            }
        }

        objects.last_mut().unwrap().next_free_idx = None;

        SlabDescriptor {
            next_free_idx: Some(unsafe { NonMaxU16::new_unchecked(0) }),
            objects,
            next,
        }
    }

    pub fn alloc_obj(&mut self, obj: T) -> &mut T {
        debug_assert!(
            self.next_free_idx.is_some(),
            "Should always be some, because if not, slab is full"
        );

        let preallocated =
            &mut self.objects[self.next_free_idx.unwrap().get() as usize];
        self.next_free_idx = unsafe { preallocated.next_free_idx };

        write_volatile!(preallocated.allocated, ManuallyDrop::new(obj));

        unsafe { &mut preallocated.allocated }
    }

    pub unsafe fn dealloc_obj(&self, obj: &T) {}
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
