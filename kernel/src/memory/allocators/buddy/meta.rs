use core::{iter::ByRefSized, ptr::NonNull};

use common::enums::BuddyOrder;

use crate::memory::{
    allocators::slab::traits::SlabPosition,
    page_descriptor::{
        NonNullPageTExt, NonNullPageUnassignedExt, Page, UnassignedPage,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct BuddyBlockMeta {
    next: Option<NonNull<UnassignedPage>>,
    prev: Option<NonNull<UnassignedPage>>,
    order: Option<BuddyOrder>,
}

impl const Default for BuddyBlockMeta {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
            order: None,
        }
    }
}

impl BuddyBlockMeta {
    pub fn detach<T: SlabPosition>(&mut self) -> Option<NonNull<Page<T>>> {
        let detached = self.next?; // None if there is no page to detach

        self.next = unsafe { detached.as_ref().buddy_meta.next };

        if let Some(mut next) = self.next {
            unsafe { next.as_mut().buddy_meta.prev = None }
        }

        Some(detached.assign::<T>())
    }

    pub fn attach<T: SlabPosition>(&mut self, mut p: NonNull<Page<T>>) {
        unsafe { p.as_mut().buddy_meta.next = self.next };

        if let Some(mut next) = self.next {
            unsafe {
                next.as_mut().buddy_meta.prev = Some(p.as_unassigned())
            };
        }

        self.next = Some(p.as_unassigned())
    }
}
