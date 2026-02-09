use core::mem::ManuallyDrop;

use buddy::meta::{BuddyPage, BuddyPageMeta};

pub union PageMeta {
    pub buddy: ManuallyDrop<BuddyPageMeta<PageMeta>>,
}

impl BuddyPage for PageMeta {
    #[inline]
    fn meta(&mut self) -> &mut BuddyPageMeta<Self> {
        unsafe { &mut self.buddy }
    }
}

// #[derive(Debug)]
// pub struct SlabPageMeta<T: Slab> {
//     pub owner: NonNull<SlabCache<T>>,
//     pub freelist: NonNull<SlabDescriptor<T>>,
// }
