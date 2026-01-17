use core::ptr::NonNull;

use common::enums::BuddyOrder;

use crate::memory::{
    allocators::slab::traits::SlabPosition,
    page_descriptor::{
        NonNullPageTExt, NonNullPageUnassignedExt, Page, UnassignedPage,
    },
};

#[derive(Default, Clone, Copy, Debug)]
pub struct BuddyBlockMeta {
    // TODO CHANGE INTO REF BECAUSE IT CONSUMES LESS MEMORY
    pub next: Option<NonNull<UnassignedPage>>,
    pub prev: Option<NonNull<UnassignedPage>>,
    pub order: Option<BuddyOrder>,
}

impl BuddyBlockMeta {
    pub fn detach<T: SlabPosition>(&mut self) -> Option<NonNull<Page<T>>> {
        let detached = self.next?; // None if there is no page to detach
        self.next = unsafe { detached.as_ref().buddy_meta.next };
        Some(detached.assign::<T>())
    }

    pub fn attach<T: SlabPosition>(&mut self, mut p: NonNull<Page<T>>) {
        unsafe { p.as_mut().buddy_meta.next = self.next };
        self.next = Some(p.as_unassigned())
    }
}
