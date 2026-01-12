use core::{
    fmt::Debug,
    mem::ManuallyDrop,
    num::{NonZeroIsize, NonZeroUsize},
    ptr::NonNull,
};

use common::{
    constants::REGULAR_PAGE_SIZE, enums::ProcessorSubClass, write_volatile,
};

use nonmax::NonMaxU16;

use crate::{alloc_pages, memory::page_descriptor::Unassigned};

impl<T> SlabDescriptor<T> {
    pub fn new(
        order: usize,
        next: Option<NonNull<SlabDescriptor<T>>>,
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
            objects: NonNull::from_mut(objects.first_mut().unwrap()),
            size: unsafe { NonZeroUsize::new_unchecked(objects.len()) },
            next,
        }
    }

    pub fn as_unassigned(&self) -> &SlabDescriptor<Unassigned> {
        unsafe {
            &*(self as *const _ as *const SlabDescriptor<Unassigned>)
        }
    }

    pub fn as_unassigned_mut(
        &mut self,
    ) -> &mut SlabDescriptor<Unassigned> {
        unsafe {
            &mut *(self as *mut _ as *mut SlabDescriptor<Unassigned>)
        }
    }
}

impl SlabDescriptor<Unassigned> {
    pub fn assign<T>(&self) -> &SlabDescriptor<T> {
        unsafe { &*(self as *const _ as *const SlabDescriptor<T>) }
    }

    pub fn assign_mut<T>(&mut self) -> &mut SlabDescriptor<T> {
        unsafe { &mut *(self as *mut _ as *mut SlabDescriptor<T>) }
    }
}

impl SlabDescriptor<SlabDescriptor<Unassigned>> {
    pub fn initial(
        order: usize,
    ) -> &'static mut SlabDescriptor<SlabDescriptor<Unassigned>> {
        let mut descriptor =
            SlabDescriptor::<SlabDescriptor<Unassigned>>::new(order, None);

        let mut d =
            descriptor.alloc_obj(descriptor.as_unassigned().clone());

        unsafe { d.as_mut().assign_mut::<SlabDescriptor<Unassigned>>() }
    }
}

pub union SlabCaches {
    pub generic4: ManuallyDrop<SlabCache<u32>>,
    pub slab_descriptor:
        ManuallyDrop<SlabDescriptor<SlabDescriptor<Unassigned>>>,
    pub slab_cache: ManuallyDrop<SlabCache<SlabCache<Unassigned>>>,
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

#[derive(Debug, Clone)]
pub struct SlabDescriptor<T: 'static + Sized> {
    /// The index in the objects array of the next free objet
    pub next_free_idx: Option<NonMaxU16>,
    pub objects: NonNull<PreallocatedObject<T>>,
    pub size: NonZeroUsize,
    pub next: Option<NonNull<SlabDescriptor<T>>>,
}

impl<T> SlabDescriptor<T> {
    pub fn object_at(&self, idx: usize) -> NonNull<PreallocatedObject<T>> {
        if idx * size_of::<T>() > self.size.get() {
            panic!("Out of bounds");
        }

        unsafe { self.objects.add(idx) }
    }

    pub fn alloc_obj(&mut self, obj: T) -> NonNull<T> {
        debug_assert!(
            self.next_free_idx.is_some(),
            "Should always be some, because if not, slab is full"
        );

        let preallocated = unsafe {
            self.object_at(self.next_free_idx.unwrap().get() as usize)
                .as_mut()
        };

        self.next_free_idx = unsafe { preallocated.next_free_idx };

        write_volatile!(preallocated.allocated, ManuallyDrop::new(obj));

        unsafe { NonNull::from_mut(&mut preallocated.allocated) }
    }

    /// Deallocate an object from this slab
    ///
    /// # Safety
    /// This function assumes that the object address is in this slab.
    pub unsafe fn dealloc_obj(&mut self, obj: *const T) {
        let freed_index = unsafe {
            self.objects
                .as_ptr()
                .offset_from(obj as *const PreallocatedObject<T>)
                as usize
        };

        unsafe {
            self.object_at(freed_index).as_mut().next_free_idx =
                self.next_free_idx
        };

        self.next_free_idx =
            unsafe { Some(NonMaxU16::new_unchecked(freed_index as u16)) };
    }
}

#[derive(Debug)]
pub struct SlabCache<T: 'static + Sized> {
    // TODO ADD LOCK
    pub buddy_order: usize,
    pub free: Option<&'static mut SlabDescriptor<T>>,
    pub partial: Option<&'static mut SlabDescriptor<T>>,
    pub full: Option<&'static mut SlabDescriptor<T>>,
}

impl<T> SlabCacheConstructor for SlabCache<T> {
    default fn new(buddy_order: usize) -> SlabCache<T> {
        unimplemented!()
    }
}

impl SlabCacheConstructor for SlabCache<SlabCache<Unassigned>> {
    fn new(buddy_order: usize) -> SlabCache<SlabCache<Unassigned>> {
        unimplemented!()
    }
}

impl SlabCache<SlabDescriptor<Unassigned>> {
    pub fn initial_cache(
        buddy_order: usize,
    ) -> SlabCache<SlabDescriptor<Unassigned>> {
        let partial =
            SlabDescriptor::<SlabDescriptor<Unassigned>>::initial(
                buddy_order,
            );

        SlabCache {
            buddy_order,
            free: None,
            partial: Some(partial),
            full: None,
        }
    }
}

trait SlabCacheConstructor {
    fn new(buddy_order: usize) -> Self;
}

const trait SlabPosition {
    fn get_position() -> usize;
}
