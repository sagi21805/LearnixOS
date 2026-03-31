use core::{mem::ManuallyDrop, ptr::NonNull};

use buddy::meta::{BuddyBlock, BuddyMeta, BuddyMetaType};

pub union PageMeta {
    pub buddy: BuddyMetaType,
}

// #[derive(Debug)]
// pub struct SlabPageMeta<T: Slab> {
//     pub owner: NonNull<SlabCache<T>>,
//     pub freelist: NonNull<SlabDescriptor<T>>,
// }
