use core::{num::NonZero, ptr::NonNull};

use common::{address_types::VirtualAddress, enums::PageSize};
use cpu_utils::structures::paging::PageEntryFlags;

use crate::memory::{
    allocators::{
        extensions::VirtualAddressExt,
        slab::{
            SLAB_ALLOCATOR,
            traits::{Slab, SlabFlags},
        },
    },
    page::{PAGES, UnassignedPage},
    unassigned::{AssignSlab, UnassignSlab, Unassigned},
};

use super::{
    descriptor::SlabDescriptor,
    traits::{SlabCacheConstructor, SlabPosition},
};

#[derive(Clone, Debug)]
pub struct SlabCache<T: 'static + Sized + SlabPosition> {
    pub buddy_order: usize,
    pub pflags: PageEntryFlags,
    pub free: Option<NonNull<SlabDescriptor<T>>>,
    pub partial: Option<NonNull<SlabDescriptor<T>>>,
    pub full: Option<NonNull<SlabDescriptor<T>>>,
}

impl<T: Slab> SlabCache<T> {
    pub fn as_unassigned(&self) -> &SlabCache<Unassigned> {
        todo!("Change to trait implementation");
        unsafe { &*(self as *const _ as *const SlabCache<Unassigned>) }
    }

    pub fn as_unassigned_mut(&mut self) -> &mut SlabCache<Unassigned> {
        todo!("Change to trait implementation");
        unsafe { &mut *(self as *mut _ as *mut SlabCache<Unassigned>) }
    }

    /// Allocate a new slab descriptor, attaches it to the free slab list,
    /// and initialize it's page.
    pub fn grow(&self) -> NonNull<SlabDescriptor<T>> {
        // Allocate a new slab descriptor for this slab
        let mut slab = unsafe {
            SLAB_ALLOCATOR
                .kmalloc::<SlabDescriptor<Unassigned>>()
                .assign::<T>()
        };

        unsafe {
            *slab.as_mut() =
                SlabDescriptor::<T>::new(self.buddy_order, None)
        }

        let slab_address: VirtualAddress =
            unsafe { slab.as_ref().objects.as_ptr().addr().into() };

        slab_address
            .set_flags(self.pflags, PageSize::Regular, unsafe {
                NonZero::<usize>::new_unchecked(1 << self.buddy_order)
            })
            .unwrap();

        let slab_page = unsafe {
            &mut PAGES[UnassignedPage::index_of_page(slab_address)]
        };

        // Set owner and freelist.
        unsafe {
            (*slab_page.meta.slab).freelist = slab.as_unassigned();
            (*slab_page.meta.slab).owner =
                NonNull::from_ref(self.as_unassigned());
        };

        slab
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
            pflags: T::PFLAGS,
            free: None,
            partial: None,
            full: None,
        }
    }
}

impl SlabCacheConstructor for SlabCache<SlabDescriptor<Unassigned>> {
    fn new(buddy_order: usize) -> SlabCache<SlabDescriptor<Unassigned>> {
        let mut partial = SlabDescriptor::<SlabDescriptor<Unassigned>>::initial_descriptor(buddy_order);

        unsafe {
            *partial.as_mut() =
                SlabDescriptor::<SlabDescriptor<Unassigned>>::new(
                    buddy_order,
                    None,
                )
        }

        let slab_address: VirtualAddress =
            unsafe { partial.as_ref().objects.as_ptr().addr().into() };

        slab_address
            .set_flags(
                SlabDescriptor::<Unassigned>::PFLAGS,
                PageSize::Regular,
                unsafe {
                    NonZero::<usize>::new_unchecked(1 << buddy_order)
                },
            )
            .unwrap();

        let slab_page = unsafe {
            &mut PAGES[UnassignedPage::index_of_page(slab_address)]
        };

        // Set owner and freelist.
        unsafe {
            (*slab_page.meta.slab).freelist = partial.as_unassigned();

            // This assumption can be made, because the created cache in
            // this function will go to the constant position on the slab
            // array defined with the `SlabPosition` array
            (*slab_page.meta.slab).owner = NonNull::from_ref(
                &SLAB_ALLOCATOR.slabs
                    [SlabDescriptor::<Unassigned>::SLAB_POSITION],
            );
        };

        SlabCache {
            buddy_order,
            pflags: SlabDescriptor::<Unassigned>::PFLAGS,
            free: None,
            partial: Some(partial),
            full: None,
        }
    }
}
