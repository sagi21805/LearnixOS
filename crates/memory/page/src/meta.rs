use core::{mem::ManuallyDrop, ptr::NonNull};

use buddy::meta::{BuddyBlock, BuddyMeta, BuddyMetaType, Regular};

pub union PageMeta {
    pub buddy: BuddyMeta<Regular>,
}

// #[derive(Debug)]
// pub struct SlabPageMeta<T: Slab> {
//     pub owner: NonNull<SlabCache<T>>,
//     pub freelist: NonNull<SlabDescriptor<T>>,
// }
