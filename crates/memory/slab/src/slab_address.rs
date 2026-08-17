use core::{num::NonZeroU64, ptr::NonNull};

use common::address_types::{Address, VirtualAddress};

use crate::{
    descriptor::SlabDescriptor,
    traits::{Slab, SlabState},
};

#[derive(Debug, Clone, Copy)]
pub struct SlabAddress(Option<NonZeroU64>);

const impl Default for SlabAddress {
    fn default() -> Self { Self(None) }
}

const impl From<u64> for SlabAddress {
    fn from(value: u64) -> Self { Self(NonZeroU64::new(value)) }
}

const impl From<SlabAddress> for u64 {
    fn from(value: SlabAddress) -> Self {
        match value.0 {
            Some(v) => v.get(),
            None => 0,
        }
    }
}

impl SlabAddress {
    pub unsafe fn as_non_null<T: Slab, S: SlabState<T>>(
        &self,
    ) -> Option<NonNull<SlabDescriptor<T, S>>> {
        unsafe {
            Some(
                VirtualAddress::new_unchecked(self.0?.get() as usize)
                    .as_non_null(),
            )
        }
    }

    pub fn from_non_null<T: Slab, S: SlabState<T>>(
        ptr: NonNull<SlabDescriptor<T, S>>,
    ) -> Self {
        Self(NonZeroU64::try_from(ptr.addr()).ok())
    }
}
