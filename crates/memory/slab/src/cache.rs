use core::ptr::NonNull;

use nonmax::NonMaxU16;

use crate::{
    descriptor::{Free, Full, Partial, PartialMeta, SlabStateKind, Used},
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

    pub fn find_partial(&self, partial: &SlabDescriptor<T, Partial>) -> Option<&SlabDescriptor<T, Partial>> {
        let mut current = self.partial?;
        while let Some(next) = unsafe { current.as_ref().next } {
            if NonNull::from_ref(partial) == current  {
                return Some(unsafe { next.as_ref() });
            }
            current = next;
        }
        None
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
        if let Some(partial) =
            self.partial.map(|mut p| unsafe { p.as_mut() })
        {
            let (allocation, final_state) = partial.alloc();
            match final_state {
                SlabStateKind::Full => {
                    self.partial = partial.next;
                    match self.full {
                        Some(mut full) => unsafe {
                            full.as_mut()
                                .attach(core::mem::transmute(partial))
                        },
                        None => {
                            self.full = Some(unsafe {
                                core::mem::transmute(partial)
                            });
                        }
                    }
                }
                _ => debug_assert!(false, "unreachable!"),
            }
            return allocation;
        }

        if let Some(free) = self.free.map(|mut p| unsafe { p.as_mut() }) {
            self.free = free.next;
            let (allocation, final_state) = free.alloc();
            let partial: &mut SlabDescriptor<T, Partial> =
                unsafe { core::mem::transmute(free) };
            match final_state {
                SlabStateKind::Partial => match self.partial {
                    Some(current_partial) => {
                        partial.next = Some(current_partial);
                    }
                    None => {}
                },
                _ => debug_assert!(false, "unreachable!"),
            }
            self.partial = Some(NonNull::from_mut(partial));
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
        slab: &mut SlabDescriptor<T, Used>,
    ) {
        let state = match slab.is_partial_mut() {
            // TODO: understand how to extract that logic into a function
            // on the slab.
            Ok(partial) => unsafe { partial.dealloc(idx) },
            Err(full) => {
                full.detach();

                let partial: &mut SlabDescriptor<T, Partial> =
                    unsafe { core::mem::transmute(full) };

                partial.state = PartialMeta::new()
                    .partial(true)
                    .next_free_idx(u16::MAX)
                    .total_allocated(T::OBJECT_PER_SLAB as u32);

                unsafe { partial.dealloc(idx) }
            }
        };

        // match state {
        //     SlabStateKind::Free => {
        //         // There should not be a lot of slabs in the partial list.
        //         // So the find cost will be small.
        //     }
        //     SlabStateKind::Partial => {}
        // }
    }
}
