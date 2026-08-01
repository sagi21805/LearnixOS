use core::ptr::NonNull;

use nonmax::NonMaxU16;

use crate::{
    descriptor::{Free, Full, Partial},
    traits::Slab,
};

use super::descriptor::SlabDescriptor;

#[derive(Debug)]
pub struct SlabCache<T: Slab> {
    pub buddy_order: usize,
    pub free: Option<NonNull<SlabDescriptor<T, Free>>>,
    pub partial: Option<NonNull<SlabDescriptor<T, Partial>>>,
    pub full: Option<NonNull<SlabDescriptor<T, Full>>>,
}

unsafe impl<T: Slab> Send for SlabCache<T> {}

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

impl SlabCache<()> {
    pub unsafe fn with<T: Slab>(&mut self) -> &mut SlabCache<T> {
        unsafe { core::mem::transmute(self) }
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
            let slab = unsafe { partial.as_mut() };

            match slab.state.is_partial() {
                Ok(partial) => {
                    let allocation = slab.alloc();
                    if partial.next_free_idx.is_none() {
                        self.partial = slab.next;

                        slab.next = self.full;
                        if let Some(mut full) = self.full {
                            full.as_mut().prev = slab.next;
                        }
                        self.full = Some(NonNull::from_mut(slab));
                    }
                    return allocation;
                }
                Err(_) => unreachable!(
                    "Found non partial state inside the partial list."
                ),
            }
        }
        if let Some(mut free) = self.free {
            let free = unsafe { free.as_mut() };

            self.free = free.next;
            free.next = self.partial;
            self.partial = Some(NonNull::from_mut(free));

            let allocation = free.alloc();
            return allocation;
        }

        todo!(
            "Handle cases where partial and free are full, and \
             allocation from the page allocator is needed."
        )
    }

    pub fn dealloc(
        &mut self,
        idx: NonMaxU16,
        slab: &mut SlabDescriptor<T>,
    ) {
        match slab.state.is_partial_mut() {
            // TODO: understand how to extract that logic into a function
            // on the slab.
            Ok(partial) => {
                unsafe {
                    slab.objects.as_mut()[idx.get() as usize]
                        .next_free_idx = partial.next_free_idx;
                }

                partial.next_free_idx = Some(idx);

                partial.flags.set_total_allocated(
                    partial.flags.get_total_allocated() - 1,
                );
            }
            Err(full_or_free) => {
                let prev =
                    unsafe { full_or_free.get_prev().as_slab_ptr::<T>() };

                if let Some(mut prev) = prev {
                    unsafe { prev.as_mut().next = slab.next };
                } else {
                    // Ensure that the prev is saved and tracks the state
                    // currectly.
                    // Because prev is `None` it is known to be the first
                    // node in either free or full.
                    debug_assert!(
                        Some(NonNull::from_mut(slab)) == self.full
                            || Some(NonNull::from_mut(slab)) == self.free
                    )
                }

                if let Some(mut next) = slab.next {
                    let next = unsafe { next.as_mut() };
                    match next.state.is_partial_mut() {
                        Ok(_) => unreachable!(
                            "Slabs in the same list should have the same \
                             state."
                        ),
                        Err(full_or_free) => {
                            let prev = unsafe {
                                full_or_free.get_prev().as_slab_ptr::<T>()
                            };

                            next.next = prev;
                        }
                    }
                }
            }
        }
    }
}
