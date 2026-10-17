use core::{num::NonZeroU64, ptr::NonNull};

use common::address_types::{Address, VirtualAddress};

use crate::{
    descriptor::{SlabDescriptor, SlabState},
    traits::Slab,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlabAddress(Option<NonZeroU64>);

// `None` takes the zero niche, so an address stays one `u64`.
const _: () = assert!(size_of::<SlabAddress>() == 8);

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
    /// The descriptor at this address, in state `S`, or `None` for no
    /// address.
    ///
    /// # Safety
    ///
    /// A non-zero address must point at a live descriptor whose metadata
    /// is `S::Meta`.
    pub unsafe fn as_non_null<T: Slab, S: SlabState>(
        &self,
    ) -> Option<NonNull<SlabDescriptor<T, S>>> {
        // SAFETY: the caller guarantees the address points at a live
        // descriptor.
        unsafe {
            Some(
                VirtualAddress::new_unchecked(self.0?.get() as usize)
                    .as_non_null(),
            )
        }
    }

    pub fn from_non_null<T: Slab, S: SlabState>(
        ptr: NonNull<SlabDescriptor<T, S>>,
    ) -> Self {
        Self(NonZeroU64::try_from(ptr.addr()).ok())
    }
}
