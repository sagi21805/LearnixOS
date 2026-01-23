use core::{mem::ManuallyDrop, ptr::NonNull};

use common::enums::BuddyOrder;

use crate::memory::{
    allocators::slab::{
        cache::SlabCache, descriptor::SlabDescriptor, traits::SlabPosition,
    },
    page::{Page, UnassignedPage},
    unassigned::{AssignSlab, UnassignSlab, Unassigned},
};

pub union PageMeta {
    pub buddy: ManuallyDrop<BuddyPageMeta>,
    pub slab: ManuallyDrop<SlabPageMeta<Unassigned>>,
}

#[derive(Debug)]
pub struct BuddyPageMeta {
    pub next: Option<NonNull<UnassignedPage>>,
    pub prev: Option<NonNull<UnassignedPage>>,
    pub order: Option<BuddyOrder>,
}

impl const Default for BuddyPageMeta {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
            order: None,
        }
    }
}

impl BuddyPageMeta {
    pub fn detach<T: SlabPosition>(&mut self) -> Option<NonNull<Page<T>>> {
        let detached = self.next?; // None if there is no page to detach

        self.next = unsafe { detached.as_ref().meta.buddy.next };

        if let Some(mut next) = self.next {
            unsafe { (*next.as_mut().meta.buddy).prev = None }
        }

        Some(detached.assign::<T>())
    }

    pub fn attach<T: SlabPosition>(&mut self, mut p: NonNull<Page<T>>) {
        unsafe { (*p.as_mut().meta.buddy).next = self.next };

        if let Some(mut next) = self.next {
            unsafe {
                (*next.as_mut().meta.buddy).prev = Some(p.as_unassigned())
            };
        }

        self.next = Some(p.as_unassigned())
    }
}

#[derive(Debug)]
pub struct SlabPageMeta<T: SlabPosition> {
    owner: NonNull<SlabCache<T>>,
    freelist: NonNull<SlabDescriptor<T>>,
}
