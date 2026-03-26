use core::{mem::ManuallyDrop, ptr::NonNull};

use buddy::meta::{BuddyBlock, BuddyMeta, BuddyMetaNode};

pub union PageMeta {
    pub buddy: ManuallyDrop<BuddyMetaNode>,
}

// #[derive(Debug)]
// pub struct SlabPageMeta<T: Slab> {
//     pub owner: NonNull<SlabCache<T>>,
//     pub freelist: NonNull<SlabDescriptor<T>>,
// }
