use core::{
    fmt::Debug,
    mem::ManuallyDrop,
    num::{NonZeroIsize, NonZeroUsize},
    ptr::NonNull,
};

use common::{
    constants::REGULAR_PAGE_SIZE, enums::ProcessorSubClass,
    late_init::LateInit, write_volatile,
};

use nonmax::NonMaxU16;

use crate::{alloc_pages, memory::page_descriptor::Unassigned};

impl<T> SlabDescriptor<T> {
    pub fn new(
        order: usize,
        next: Option<NonNull<SlabDescriptor<T>>>,
    ) -> SlabDescriptor<T> {
        let address = unsafe { alloc_pages!(1 << order) };
        let mut objects = unsafe {
            NonNull::slice_from_raw_parts(
                NonNull::new_unchecked(
                    address as *mut PreallocatedObject<T>,
                ),
                ((1 << order) * REGULAR_PAGE_SIZE) / size_of::<T>(),
            )
        };

        for (i, object) in
            unsafe { objects.as_mut() }.iter_mut().enumerate()
        {
            *object = PreallocatedObject {
                next_free_idx: Some(unsafe {
                    NonMaxU16::new_unchecked(i as u16 + 1)
                }),
            }
        }

        unsafe {
            objects.as_mut().last_mut().unwrap().next_free_idx = None
        };

        SlabDescriptor {
            next_free_idx: Some(unsafe { NonMaxU16::new_unchecked(0) }),
            objects,
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
    pub fn initial_descriptor(
        order: usize,
    ) -> NonNull<SlabDescriptor<SlabDescriptor<Unassigned>>> {
        let mut descriptor =
            SlabDescriptor::<SlabDescriptor<Unassigned>>::new(order, None);

        let mut d =
            descriptor.alloc_obj(descriptor.as_unassigned().clone());

        unsafe {
            NonNull::from_mut(
                d.as_mut().assign_mut::<SlabDescriptor<Unassigned>>(),
            )
        }
    }
}

pub union SlabCaches {
    pub generic4: ManuallyDrop<SlabCache<u32>>,
    pub slab_descriptor:
        ManuallyDrop<SlabCache<SlabDescriptor<Unassigned>>>,
    pub slab_cache: ManuallyDrop<SlabCache<SlabCache<Unassigned>>>,
    pub uninit: (),
}

macro_rules! define_slab_system {
    ($($t:ty),* $(,)?) => {
        // 1. Implement the trait for each type
        register_slabs!($($t),*);

        // 2. Calculate count
        const COUNT: usize = [$(stringify!($t)),*].len();

        // 3. Create the static array
        pub static SLABS: [SlabCaches; COUNT] = [
            $(
                // We mention $t inside a block but don't actually use it.
                // This tells Rust: "Repeat this block for every type in $t"
                {
                    stringify!($t);
                    SlabCaches { uninit: () }
                }
            ),*
        ];
    }
}

macro_rules! register_slabs {
    // 1. Entry point: handle trailing commas by calling the internal @step
    ($($t:ty),* $(,)?) => {
        register_slabs!(@step 0; $($t),*);
    };

    // 2. The recursive step: Matches a type, a comma, and at least one more type
    (@step $idx:expr; $head:ty, $($tail:ty),+) => {
        impl SlabPosition for $head {
            const POSITION: usize = $idx;
        }
        register_slabs!(@step $idx + 1; $($tail),*);
    };

    // 3. The base case: Matches exactly one last type (no trailing comma)
    (@step $idx:expr; $head:ty) => {
        impl SlabPosition for $head {
            const POSITION: usize = $idx;
        }
    };

    // 4. The empty case: If someone calls it with nothing
    (@step $idx:expr; ) => {};
}
define_slab_system!(SlabDescriptor<Unassigned>,);

unsafe impl<T> Send for SlabDescriptor<T> {}
unsafe impl<T> Sync for SlabDescriptor<T> {}

unsafe impl Send for SlabCaches {}
unsafe impl Sync for SlabCaches {}

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
    pub objects: NonNull<[PreallocatedObject<T>]>,
    pub next: Option<NonNull<SlabDescriptor<T>>>,
}

impl<T> SlabDescriptor<T> {
    pub fn alloc_obj(&mut self, obj: T) -> NonNull<T> {
        debug_assert!(
            self.next_free_idx.is_some(),
            "Should always be some, because if not, slab is full"
        );

        let preallocated = unsafe {
            &mut self.objects.as_mut()
                [self.next_free_idx.unwrap().get() as usize]
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
        let freed_index =
            (obj.addr() - self.objects.as_ptr().addr()) / size_of::<T>();

        unsafe {
            self.objects.as_mut()[freed_index].next_free_idx =
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
    pub free: Option<NonNull<SlabDescriptor<T>>>,
    pub partial: Option<NonNull<SlabDescriptor<T>>>,
    pub full: Option<NonNull<SlabDescriptor<T>>>,
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
            SlabDescriptor::<SlabDescriptor<Unassigned>>::initial_descriptor(
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

/// Get the position on the slab array, for a slab of the given type.
///
/// Shouldn't implement this trait manually, and it is implemented once
/// with a macro.
pub const trait SlabPosition {
    const POSITION: usize;
}
