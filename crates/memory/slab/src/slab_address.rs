use core::{num::NonZeroU64, ptr::NonNull};

use common::address_types::{Address, VirtualAddress};

use crate::{
    descriptor::{SlabDescriptor, SlabState},
    traits::Slab,
};

#[repr(transparent)]
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
    /// A pointer to the descriptor at this address, in state `S`, or
    /// `None` for no address. Dereferencing it needs a live descriptor
    /// whose metadata is `S::Meta` at this address.
    //
    // TODO: rebuild links as `base.with_addr(address)`, from one pointer
    // that covers all descriptor memory, so Miri can check them under
    // `-Zmiri-strict-provenance`. Exposed provenance hides some bugs from
    // Miri, such as a link into the wrong allocation.
    pub fn as_non_null<T: Slab, S: SlabState>(
        self,
    ) -> Option<NonNull<SlabDescriptor<T, S>>> {
        Some(VirtualAddress::from(self.0?.get() as usize).as_non_null())
    }

    /// The descriptor at this address, in state `S`, or `None` for no
    /// address.
    ///
    /// # Safety
    ///
    /// A non-zero address must be a list head, or the `next` or `prev`
    /// of a node, in a list the caller has locked, so a live descriptor in
    /// state `S` sits there. The caller must hold no other reference
    /// to that descriptor while this one is used.
    pub(crate) unsafe fn node_mut<'a, T: Slab, S: SlabState>(
        self,
    ) -> Option<&'a mut SlabDescriptor<T, S>> {
        self.as_non_null().map(|mut node| unsafe { node.as_mut() })
    }

    /// The address of the descriptor at `ptr`, with its provenance exposed
    /// so [`Self::as_non_null`] rebuilds a pointer that may access it.
    pub fn from_non_null<T: Slab, S: SlabState>(
        ptr: NonNull<SlabDescriptor<T, S>>,
    ) -> Self {
        Self(NonZeroU64::try_from(ptr.expose_provenance()).ok())
    }
}
