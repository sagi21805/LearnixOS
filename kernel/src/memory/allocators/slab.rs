use core::{fmt::Debug, mem::ManuallyDrop};

use common::{constants::REGULAR_PAGE_SIZE, write_volatile};

use nonmax::NonMaxU16;

use crate::{alloc_pages, memory::page_descriptor::Unassigned};

trait SlabConstructor<T> {
    fn new(
        order: usize,
        next: Option<&'static mut SlabDescriptor<T>>,
    ) -> Self;
}

impl<T> SlabConstructor<T> for SlabDescriptor<T> {
    default fn new(
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
}

impl SlabConstructor<SlabDescriptor<Unassigned>>
    for SlabDescriptor<Unassigned>
{
    fn new(
        _order: usize,
        _next: Option<
            &'static mut SlabDescriptor<SlabDescriptor<Unassigned>>,
        >,
    ) -> Self {
        unimplemented!()
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

impl SlabDescriptor<SlabDescriptor<Unassigned>> {
    pub fn new() {}
}

#[derive(Debug)]
pub struct SlabDescriptor<T: 'static + Sized> {
    /// The index in the objects array of the next free objet
    pub next_free_idx: Option<NonMaxU16>,
    pub objects: &'static mut [PreallocatedObject<T>],
    pub next: Option<&'static mut SlabDescriptor<T>>,
}

impl<T> SlabDescriptor<T> {
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

    /// Deallocate an object from this slab
    ///
    /// # Safety
    /// This function assumes that the object address is in this slab.
    pub unsafe fn dealloc_obj(&mut self, obj: &T) {
        let freed_index = unsafe {
            self.objects.as_ptr().offset_from(
                obj as *const _ as *const PreallocatedObject<T>,
            ) as usize
        };

        self.objects[freed_index].next_free_idx = self.next_free_idx;

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

impl<T> SlabCache<T> {
    pub const fn new() -> SlabCache<T> {
        unimplemented!()
    }
}

impl SlabCache<SlabCache<Unassigned>> {
    pub fn initial(
        buddy_order: usize,
    ) -> SlabCache<SlabCache<Unassigned>> {
        let partial = SlabDescriptor::<SlabCache<Unassigned>>::new(
            buddy_order,
            None,
        );
        let full = SlabDescriptor::<SlabCache<Unassigned>>::new(
            buddy_order,
            None,
        );
    }
}

const trait SlabPosition {
    fn get_position() -> usize;
}
