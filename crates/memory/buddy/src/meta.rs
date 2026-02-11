use core::ptr::NonNull;

use common::enums::BuddyOrder;

pub trait BuddyPage {
    fn meta(&mut self) -> &mut BuddyPageMeta<Self>;
}

#[derive(Debug)]
pub struct BuddyPageMeta<T: BuddyPage + ?Sized> {
    pub next: Option<NonNull<T>>,
    pub prev: Option<NonNull<T>>,
    pub order: Option<BuddyOrder>,
}

impl<T: BuddyPage> const Default for BuddyPageMeta<T> {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
            order: None,
        }
    }
}

impl<T: BuddyPage> BuddyPageMeta<T> {
    pub fn detach(&mut self) -> Option<NonNull<T>> {
        let mut detached = self.next?; // None if there is no page to detach

        self.next = unsafe { detached.as_mut().meta().next };

        if let Some(mut next) = self.next {
            unsafe { next.as_mut().meta().prev = None }
        }

        Some(detached)
    }

    pub fn attach(&mut self, mut p: NonNull<T>) {
        unsafe { p.as_mut().meta().next = self.next };

        if let Some(mut next) = self.next {
            unsafe { next.as_mut().meta().prev = Some(p) };
        }

        self.next = Some(p)
    }
}
