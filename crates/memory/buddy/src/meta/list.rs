//! Linking blocks into and out of a free list.

use core::ptr::NonNull;

use crate::meta::{BuddyBlock, BuddyMeta, MetaState, Regular};

impl<S> BuddyMeta<S>
where
    S: MetaState<Next = Option<NonNull<BuddyMeta<Regular>>>>,
{
    #[inline]
    pub fn attach(&mut self, mut p: NonNull<BuddyMeta<Regular>>) {
        unsafe { p.as_mut().next = self.next };
        if let Some(mut next) = self.next {
            unsafe { next.as_mut().prev = p.cast() };
        }
        self.next = Some(p.cast());
        unsafe { p.as_mut().prev = NonNull::from_mut(self).cast() };
    }

    #[inline]
    pub fn attach_block<Block: const BuddyBlock>(
        &mut self,
        mut p: NonNull<Block>,
    ) {
        self.attach(NonNull::from_mut(unsafe { p.as_mut().meta_mut() }));
    }
}

impl BuddyMeta<Regular> {
    /// Detaches self from the list.
    pub fn detach(&mut self) -> NonNull<BuddyMeta<Regular>> {
        unsafe { self.prev.as_mut().next = self.next }

        if let Some(mut next) = self.next {
            unsafe { next.as_mut().prev = self.prev };
        }

        NonNull::from_mut(self)
    }
}
