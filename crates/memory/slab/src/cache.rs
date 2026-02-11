use core::{num::NonZero, ptr::NonNull};

use common::address_types::VirtualAddress;

use crate::{traits::Slab, unassigned::UnassignSlab};

use super::{descriptor::SlabDescriptor, traits::SlabCacheConstructor};

#[derive(Clone, Debug)]
pub struct SlabCache<T: Slab> {
    pub buddy_order: usize,
    pub free: Option<NonNull<SlabDescriptor<T>>>,
    pub partial: Option<NonNull<SlabDescriptor<T>>>,
    pub full: Option<NonNull<SlabDescriptor<T>>>,
}

impl<T: Slab> UnassignSlab for NonNull<SlabCache<T>> {
    type Target = NonNull<SlabCache<()>>;

    fn as_unassigned(&self) -> Self::Target {
        unsafe {
            NonNull::new_unchecked(self.as_ptr() as *mut SlabCache<()>)
        }
    }
}

impl<T: Slab> SlabCache<T> {
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

    pub fn take_ownership(&self, slab: NonNull<SlabDescriptor<T>>) {
        let slab_address: VirtualAddress =
            unsafe { slab.as_ref().objects.as_ptr().addr().into() };

        slab_address
            .set_flags(T::PFLAGS, T::PSIZE, unsafe {
                NonZero::<usize>::new_unchecked(1 << self.buddy_order)
            })
            .unwrap();

        let slab_page =
            unsafe { UnassignedPage::from_virt(slab_address).as_mut() };

        // Set owner and freelist.
        unsafe {
            (*slab_page.meta.slab).freelist = slab.as_unassigned();
            (*slab_page.meta.slab).owner =
                NonNull::from_ref(self).as_unassigned();
        };
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

impl SlabCache<()> {
    pub fn assign<T: Slab>(&self) -> NonNull<SlabCache<T>> {
        unsafe {
            NonNull::new_unchecked(self as *const _ as *mut SlabCache<T>)
        }
    }
}

impl<T: Slab> SlabCacheConstructor for SlabCache<T> {
    default fn new(buddy_order: usize) -> SlabCache<T> {
        SlabCache {
            buddy_order,
            free: None,
            partial: None,
            full: None,
        }
    }
}

impl SlabCacheConstructor for SlabCache<SlabDescriptor<()>> {
    fn new(buddy_order: usize) -> SlabCache<SlabDescriptor<()>> {
        let partial =
            SlabDescriptor::<SlabDescriptor<()>>::initial_descriptor(
                buddy_order,
            );

        // This assumption can be made, because the created cache in
        // this function will go to the constant position on the slab
        // array defined with the `SlabPosition` array
        let mut future_owner =
            unsafe { SLAB_ALLOCATOR.slab_of::<SlabDescriptor<()>>() };

        let cache = SlabCache {
            buddy_order,
            free: None,
            partial: Some(partial),
            full: None,
        };

        // Only in this function, we initialiuze the global array in the
        // new function.
        //
        // Because then we can use the `take_ownership` function
        unsafe {
            *future_owner.as_mut() = cache.clone();
            future_owner.as_mut().take_ownership(partial);
        }

        cache
    }
}
